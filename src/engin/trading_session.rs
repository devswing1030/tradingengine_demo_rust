use crate::{order_book::*, messages::NewOrder};
use crate::messages::{CancelRequest, OrigOrderInfoForCancel, ExecutionTask, OrderMatchedInfo, RcResult};
use crate::order_book::auction_order::*;
use crate::types::*;
use crate::auction::continuos::Continuos;
use crate::auction::continuos::TradingSessionData;

use std::sync::Arc;
use std::rc::Rc;

pub struct NewOrderForBook {
    order : Arc<NewOrder>,
    rc_info : Box<RcResult>
} 
impl AuctionOrder for NewOrderForBook {
    fn qty(&self) -> Qty{
        self.order.qty
    }
    fn price(&self) -> Price{
        self.order.price
    }
    fn order_id(&self) -> OrderID{
        self.order.order_id
    }
}

pub struct TradingSession {
    buy_order_book : PriceOrderBook<NewOrderForBook>,
    sell_order_book : PriceOrderBook<NewOrderForBook>,
}

impl TradingSessionData<NewOrderForBook> for TradingSession {
    fn get_buy_order_book(&mut self) -> &mut PriceOrderBook<NewOrderForBook> {
        &mut self.buy_order_book
    }
    fn get_sell_order_book(&mut self) -> &mut PriceOrderBook<NewOrderForBook> {
        &mut self.sell_order_book
    }
}

impl TradingSession {
    pub fn new() -> TradingSession {
        TradingSession {  
            buy_order_book : PriceOrderBook::create_high_price_priority_order_book(),
            sell_order_book : PriceOrderBook::create_low_price_priority_order_book(),
        }
    }
    pub fn process_new_order(&mut self, order : Arc<NewOrder>, rc_info : Box<RcResult>) -> Vec<ExecutionTask> {
        let mut c = Continuos::<NewOrderForBook> { session : self};
        let side = order.side;
        let tmp = Rc::new(NewOrderForBook {order:order.clone(), rc_info : rc_info});
        let consumed_orders = c.process_new_order(side, tmp);

        let mut tasks = Vec::new();
        let mut leaves_qty = order.qty;
        consumed_orders.iter().for_each(|contra| {
            leaves_qty -= contra.consumed_qty;
            tasks.push(ExecutionTask::NewoOrderMatched(
                OrderMatchedInfo {
                    order1 : order.clone(),
                    leaves_qty1 : leaves_qty,
                    order2 : contra.orig_order.order.clone(),
                    leaves_qty2 : contra.leaves_qty,
                    last_px : contra.orig_order.price(),
                    last_qty : contra.consumed_qty
                }
            ))
        });
        tasks
    }
    pub fn process_cancel_request(&mut self, orig_info : &OrigOrderInfoForCancel, cancel_request : Box<CancelRequest>) -> ExecutionTask {
        let mut c = Continuos::<NewOrderForBook> { session : self};
        if let Some(orig) = c.process_cancel_request(orig_info) {
            ExecutionTask::CancelRequestAccepted(orig.consumed_qty, cancel_request, orig.orig_order.order.clone())
        }
        else {
            ExecutionTask::CancelRequestRejected(CancelReasonCode::OrderNotExisted, cancel_request)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{types::*, messages::NewOrder};
    use crate::messages::{RcResult, ExecutionTask};

    use super::TradingSession;


    struct OrderGen {
        order_id : OrderID,
    }

    impl OrderGen {
        fn new() -> OrderGen {
            OrderGen { order_id: 0 }
        }
        fn gen_order(&mut self, side : Side, price : Price, qty : Qty) -> Arc<NewOrder> {
            self.order_id += 1;

            Arc::new(NewOrder { 
                order_id: self.order_id,
                pbu_id: to_array("PBU001"), 
                cl_ord_id: to_array(""),
                security_id: to_array(""), 
                side: side, 
                price: price, 
                qty: qty
            })
        }
    }

    fn assert_order_matched_execution(task : &ExecutionTask, last_px : Price, last_qty : Qty, leaves_qty1 : Qty, leaves_qty2 : Qty) {
        match task {
            ExecutionTask::NewoOrderMatched(info) => {
                assert_eq!(info.last_px, last_px);
                assert_eq!(info.last_qty, last_qty);
                assert_eq!(info.leaves_qty1, leaves_qty1);
                assert_eq!(info.leaves_qty2, leaves_qty2);
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn test_new_order() {
        let mut gen = OrderGen::new();
        let mut session = TradingSession::new();

        let order = gen.gen_order(K_BUY, 20, 50);
        session.process_new_order(order, Box::new(RcResult{}));
        let order = gen.gen_order(K_BUY, 30, 50);
        session.process_new_order(order, Box::new(RcResult{}));
        let order = gen.gen_order(K_BUY, 40, 50);
        session.process_new_order(order, Box::new(RcResult{}));

        let order = gen.gen_order(K_SELL, 15, 120);
        let consumed_orders = session.process_new_order(order, Box::new(RcResult{}));
        assert_order_matched_execution(&consumed_orders[0], 40, 50, 70, 0);
        assert_order_matched_execution(&consumed_orders[1], 30, 50, 20, 0);
        assert_order_matched_execution(&consumed_orders[2], 20, 20, 0, 30);

    }

}