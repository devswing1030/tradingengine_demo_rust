use crate::messages::*;

pub struct RcProcessor {

}

impl RcProcessor {
    pub fn new() -> RcProcessor {
        RcProcessor{}
    }

    pub fn process(&mut self, task : RcProcessorTask) -> CoreProcessorTask {
        match task {
            RcProcessorTask::NewOrder(order) => { self.process_new_order(order) },
            RcProcessorTask::NewOrderRejected(info) => CoreProcessorTask::NewOrderRejected(info),
            RcProcessorTask::CancelRequest(info, cancel_request) => CoreProcessorTask::CancelRequest(info, cancel_request),
            RcProcessorTask::CancelRequestRejected(info) => CoreProcessorTask::CancelRequestRejected(info),
        }
    }

    pub fn process_new_order(&mut self, order : Box<NewOrder>) -> CoreProcessorTask {
        CoreProcessorTask::NewOrder( order, Box::new(RcResult{}))
    }
}