use crate::messages::*;
use std::sync::Arc;
use crate::engin::trading_session::TradingSession;

pub struct CoreProcessor {
    session : TradingSession,
}

impl CoreProcessor {
    pub fn new() -> CoreProcessor {
        CoreProcessor {
            session : TradingSession::new(),
        }
    }

    pub fn process<F>(&mut self, task : CoreProcessorTask, mut exe_gen : F ) 
          where F : FnMut(ExecutionTask) {
        match task {
            CoreProcessorTask::NewOrder(order, rc_info) => self.process_new_order(order, rc_info, exe_gen),
            CoreProcessorTask::NewOrderRejected(info) => exe_gen(ExecutionTask::NewOrderRejected(info)),
            CoreProcessorTask::CancelRequest(info, cancle_request) => self.process_cancel_request(info, cancle_request, exe_gen),
            CoreProcessorTask::CancelRequestRejected(info) => exe_gen(ExecutionTask::CancelRequestRejected(info.0, info.1)),
        }
    }

    fn process_new_order<F>(&mut self, order : Box<NewOrder>, rc_info : Box<RcResult>, mut exe_gen : F)
         where F : FnMut(ExecutionTask) {

        let order = Arc::from(order);
        exe_gen(ExecutionTask::NewOrderAccepted(Arc::clone(&order)));

        self.session.process_new_order(order, rc_info).into_iter().for_each(|task| {
            exe_gen(task);
         });
    }

    fn process_cancel_request<F>(&mut self, orig_info : OrigOrderInfoForCancel, cancel_request : Box<CancelRequest>, mut exe_gen : F) 
         where F : FnMut(ExecutionTask) {
        exe_gen(self.session.process_cancel_request(&orig_info, cancel_request));
    }
}