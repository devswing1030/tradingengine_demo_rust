use std::collections::BTreeMap;
use std::collections::BTreeSet;
use crate::types::CancelReasonCode;
use crate::types::*;
use crate::messages::*;

pub struct PreProcessor {
    new_orders : BTreeMap<(PBUID, ClOrdID), OrigOrderInfoForCancel>,
    cancel_requests : BTreeSet<(PBUID, ClOrdID)>
}
impl PreProcessor {
    pub fn new() -> PreProcessor {
        PreProcessor {
            new_orders : BTreeMap::new(),
            cancel_requests : BTreeSet::new()
        }
    }

    pub fn process(&mut self, task: PreProcessorTask) -> RcProcessorTask{
        match task {
            PreProcessorTask::NewOrder(new_order) => { self.process_new_order(new_order) },
            PreProcessorTask::CancelRequest(cancel_request) => { self.process_cancel_request(cancel_request) }
        }
    }

    fn process_new_order(&mut self, new_order : Box<NewOrder>) -> RcProcessorTask {
        if self.cancel_requests.contains(&(new_order.pbu_id.clone(), new_order.cl_ord_id.clone())) {
            return RcProcessorTask::NewOrderRejected((CancelReasonCode::Duplicated, new_order));
        }

        let mut duplicated = false;
        self.new_orders.entry((new_order.pbu_id.clone(), new_order.cl_ord_id.clone()))
           .and_modify(|_v| {
                duplicated = true;
            })
            .or_insert(new_order.get_info_for_cancel());
        
        if duplicated {
            RcProcessorTask::NewOrderRejected((CancelReasonCode::Duplicated, new_order))
        }
        else {
            RcProcessorTask::NewOrder(new_order)
        }
    }

    fn process_cancel_request(&mut self, cancel_request : Box<CancelRequest>) -> RcProcessorTask {
        if self.new_orders.contains_key(&(cancel_request.pbu_id.clone(), cancel_request.cl_ord_id.clone())) {
            return RcProcessorTask::CancelRequestRejected((CancelReasonCode::Duplicated, cancel_request));
        }
        
        if !self.cancel_requests.insert((cancel_request.pbu_id.clone(), cancel_request.cl_ord_id.clone())) {
            return RcProcessorTask::CancelRequestRejected((CancelReasonCode::Duplicated, cancel_request));
        }

        if let Some(info) = self.new_orders.get(&(cancel_request.pbu_id.clone(), cancel_request.orig_cl_ord_id.clone())) {
            if cancel_request.security_id != info.security_id {
                return RcProcessorTask::CancelRequestRejected((CancelReasonCode::InvalidSecurity, cancel_request));
            }
            return RcProcessorTask::CancelRequest(info.clone(), cancel_request);
        }
        else {
            return RcProcessorTask::CancelRequestRejected((CancelReasonCode::OrderNotExisted, cancel_request));
        }
    }
    
}


#[cfg(test)]

mod tests {
    use crate::{messages::{NewOrder, PreProcessorTask, RcProcessorTask, CancelRequest}, types::{CancelReasonCode, to_array}};

    use super::PreProcessor;


    fn assert_cancel_reason(task : &RcProcessorTask, code : CancelReasonCode) {
        match task {
        RcProcessorTask::NewOrderRejected((tmp_code, _)) => {
            if code == *tmp_code {
            }
            else {
                assert!(false);
            }
        },
        RcProcessorTask::CancelRequestRejected((tmp_code, _)) => {
            if code == *tmp_code {
            }
            else {
                assert!(false);
            }
        },
        _ => assert!(false),
        }
    }

    #[test]
    fn test_new_order() {
        let mut p = PreProcessor::new();

        let order = Box::new(
            NewOrder {
                pbu_id: to_array("000100"), 
                cl_ord_id:to_array("123"),
                order_id : 0,
                security_id : to_array("SEC001"),
                price : 100,
                qty : 100,
                side : 'B'
        });
        let task = PreProcessorTask::NewOrder(order);
        let task = p.process(task);
        if let RcProcessorTask::NewOrder(_) = task {
        }
        else {
            assert!(false)
        }

        let order = Box::new(
            NewOrder {
                pbu_id: to_array("000100"), 
                cl_ord_id: to_array("124"),
                order_id : 1,
                security_id : to_array("SEC001"),
                price : 110,
                qty : 100,
                side : 'B'
        });
        let task = PreProcessorTask::NewOrder(order);
        let task = p.process(task);
        if let RcProcessorTask::NewOrder(_) = task {
        }
        else {
            assert!(false)
        }

        let order = Box::new(
            NewOrder {
                pbu_id: to_array("000100"), 
                cl_ord_id:to_array("123"),
                order_id : 0,
                security_id : to_array("SEC001"),
                price : 100,
                qty : 100,
                side : 'B'
        });
        let task = PreProcessorTask::NewOrder(order);
        let task = p.process(task);
        assert_cancel_reason(&task, CancelReasonCode::Duplicated);

        let cancel = Box::new(
            CancelRequest {
                pbu_id: to_array("000100"), 
                cl_ord_id: to_array("123"),
                order_id : 0,
                security_id : to_array("SEC001"),
                orig_cl_ord_id : to_array("124")
            }
        );
        let task = PreProcessorTask::CancelRequest(cancel);
        let task = p.process(task);
        assert_cancel_reason(&task, CancelReasonCode::Duplicated);

        let cancel = Box::new(
            CancelRequest {
                pbu_id: to_array("000100"), 
                cl_ord_id: to_array("125"),
                order_id : 0,
                security_id : to_array("SEC002"),
                orig_cl_ord_id : to_array("124")
            }
        );
        let task = PreProcessorTask::CancelRequest(cancel);
        let task = p.process(task);
        assert_cancel_reason(&task, CancelReasonCode::InvalidSecurity);

        let cancel = Box::new(
            CancelRequest {
                pbu_id: to_array("000100"), 
                cl_ord_id: to_array("125"),
                order_id : 0,
                security_id : to_array("SEC001"),
                orig_cl_ord_id : to_array("124")
            }
        );
        let task = PreProcessorTask::CancelRequest(cancel);
        let task = p.process(task);
        assert_cancel_reason(&task, CancelReasonCode::Duplicated);

        let cancel = Box::new(
            CancelRequest {
                pbu_id: to_array("000100"), 
                cl_ord_id: to_array("126"),
                order_id : 0,
                security_id : to_array("SEC001"),
                orig_cl_ord_id : to_array("124")
            }
        );
        let task = PreProcessorTask::CancelRequest(cancel);
        let task = p.process(task);
        if let RcProcessorTask::CancelRequest(info, _) = task {
            assert_eq!(info.security_id, to_array("SEC001"));
            assert_eq!(info.price, 110);
            assert_eq!(info.order_id, 1);
            assert_eq!(info.side, crate::types::K_BUY);
        } 
        else {
            assert!(false);
        }

        let order = Box::new(
            NewOrder {
                pbu_id: to_array("000100"), 
                cl_ord_id: to_array("126"),
                order_id : 0,
                security_id : to_array("SEC001"),
                price : 100,
                qty : 100,
                side : 'B'
        });
        let task = PreProcessorTask::NewOrder(order);
        let task = p.process(task);
        assert_cancel_reason(&task, CancelReasonCode::Duplicated);



    }
}
