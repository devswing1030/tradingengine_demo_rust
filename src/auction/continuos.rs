use crate::messages::OrigOrderInfoForCancel;
use crate::order_book::PriceOrderBook;
use crate::types::*;
use crate::order_book::auction_order::AuctionOrder;
use crate::order_book::auction_order::ConsumedOrder;

use std::rc::Rc;

pub trait TradingSessionData<Order> {
    fn get_buy_order_book(&mut self) -> &mut PriceOrderBook<Order>;
    fn get_sell_order_book(&mut self) -> &mut PriceOrderBook<Order>;
}

pub struct Continuos<'a, Order> {
    pub session : &'a mut dyn TradingSessionData<Order>,
}

impl <Order : AuctionOrder> Continuos<'_, Order> {
    pub fn process_new_order(&mut self, side : Side, order : Rc<Order>) -> Vec<ConsumedOrder<Order>>
    {
        let re;
        if side == K_BUY {
            re = Continuos::native_process_new_order(&order, self.session.get_sell_order_book());
            Continuos::insert_order(order, re.0, self.session.get_buy_order_book());
        }
        else if side == K_SELL {
            re = Continuos::native_process_new_order(&order, self.session.get_buy_order_book());
            Continuos::insert_order(order, re.0, self.session.get_sell_order_book());
        }
        else {
            panic!("Invalid Order Side");
        }
        re.1
    }

    pub fn process_cancel_request(&mut self, orig_info : &OrigOrderInfoForCancel) -> Option<ConsumedOrder<Order>>{
        if orig_info.side == K_BUY {
            self.session.get_buy_order_book().remove_order(orig_info.price, orig_info.order_id)
        }
        else if orig_info.side == K_SELL {
            self.session.get_sell_order_book().remove_order(orig_info.price, orig_info.order_id)
        }
        else {
            panic!("Invalid Order Side");
        }
    }

    fn native_process_new_order(order : &Rc<Order>, contra_book : &mut PriceOrderBook<Order>) -> (Qty, Vec<ConsumedOrder<Order>>) {
        contra_book.consume_order(order.qty(), order.price())
    }

    fn insert_order(order : Rc<Order>, leaves_qty : Qty, book : &mut PriceOrderBook<Order>) {
        if leaves_qty > 0 {
            book.insert_order_with_leaves_qty(leaves_qty, order);
        }
    }
}
