use crate::types::*;
use std::sync::Arc;


#[derive(Debug)]
pub struct NewOrder {
    pub order_id : OrderID,
    pub pbu_id : PBUID,
    pub cl_ord_id : ClOrdID,
    pub security_id : SecurityID,
    pub side : Side,
    pub price : Price,
    pub qty : Qty,
}

impl NewOrder {
    pub fn get_info_for_cancel(&self) -> OrigOrderInfoForCancel {
        OrigOrderInfoForCancel { security_id: self.security_id.clone(), 
            order_id: self.order_id.clone(), side: self.side.clone(), price: self.price.clone() }
    }
}

#[derive(Debug)]
pub struct CancelRequest {
    pub order_id : OrderID,
    pub pbu_id : PBUID,
    pub cl_ord_id : ClOrdID,
    pub orig_cl_ord_id : ClOrdID,
    pub security_id : SecurityID,
}

#[derive(Debug)]
pub struct OrigOrderInfoForCancel {
    pub security_id : SecurityID,
    pub order_id : OrderID,
    pub side : Side,
    pub price : Price,
}
impl OrigOrderInfoForCancel {
    pub fn clone(&self) -> OrigOrderInfoForCancel {
        OrigOrderInfoForCancel { security_id: self.security_id.clone(), 
            order_id: self.order_id.clone(), side: self.side.clone(), price: self.price.clone() }
    }
}
#[derive(Debug)]
pub enum PreProcessorTask {
    NewOrder(Box<NewOrder>),
    CancelRequest(Box<CancelRequest>),
}

#[derive(Debug)]
pub enum RcProcessorTask {
    NewOrder(Box<NewOrder>),
    NewOrderRejected((CancelReasonCode, Box<NewOrder>)),
    CancelRequest(OrigOrderInfoForCancel, Box<CancelRequest>),
    CancelRequestRejected((CancelReasonCode, Box<CancelRequest>))
}

#[derive(Debug)]
pub struct RcResult;

#[derive(Debug)]
pub enum CoreProcessorTask {
    NewOrder(Box<NewOrder>, Box<RcResult>),
    NewOrderRejected((CancelReasonCode, Box<NewOrder>)),
    CancelRequest(OrigOrderInfoForCancel, Box<CancelRequest>),
    CancelRequestRejected((CancelReasonCode, Box<CancelRequest>))
}

 
#[derive(Debug)]
pub struct OrderMatchedInfo {
    pub order1 : Arc<NewOrder>,
    pub leaves_qty1 : Qty,
    pub order2 : Arc<NewOrder>,
    pub leaves_qty2 : Qty,
    pub last_px : Price,
    pub last_qty : Qty
}

#[derive(Debug)]
pub enum ExecutionTask {
    NewOrderAccepted(Arc<NewOrder>),
    NewOrderRejected((CancelReasonCode, Box<NewOrder>)),
    CancelRequestAccepted(Qty/*leaves_qty */, Box<CancelRequest>, Arc<NewOrder>),
    CancelRequestRejected(CancelReasonCode, Box<CancelRequest>),
    NewoOrderMatched(OrderMatchedInfo)
}


#[cfg(test)]
    use std::rc::Rc;

    pub struct OrderGen {
        order_id : OrderID,
    }

    impl OrderGen {
        pub fn new() -> OrderGen {
            OrderGen { order_id: 0 }
        }
        pub fn gen_order(&mut self, side : Side, price : Price, qty : Qty) -> Box<NewOrder> {
            self.order_id += 1;

            Box::new(NewOrder { 
                order_id: self.order_id,
                pbu_id: to_array("PBU001"), 
                cl_ord_id: to_array(&self.order_id.to_string()),
                security_id: to_array(""), 
                side: side, 
                price: price, 
                qty: qty
            })
        }
        pub fn get_cancel_request(&mut self, orig_order : &Box<NewOrder>) -> Box<CancelRequest> {
            self.order_id += 1;
            Box::new(CancelRequest { order_id: self.order_id, 
                pbu_id: orig_order.pbu_id.clone(), 
                cl_ord_id: to_array(&self.order_id.to_string()),
                orig_cl_ord_id: orig_order.cl_ord_id.clone(), 
                security_id: orig_order.security_id.clone() })
        }
    }
