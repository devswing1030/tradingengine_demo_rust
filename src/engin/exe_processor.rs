use crate::messages::*;
use crate::types::*;

use serde::Serialize;
use bincode;


pub struct ExeProcessor {
    exec_id : ExecID,
}

impl ExeProcessor {
    pub fn new() -> ExeProcessor {
        ExeProcessor { exec_id: 0 }
    }

    pub fn process(&mut self, task : ExecutionTask, sender : &mut ExeSender) {
        match task {
            ExecutionTask::NewOrderAccepted(order) => {
                self.exec_id += 1;
                let mut report = new_order_accepted(order.as_ref());
                report.exec_id = self.exec_id;
                sender.send(bincode::serialize(&report).unwrap());
            },
            ExecutionTask::NewOrderRejected((reason, order)) => {
                self.exec_id += 1;
                let mut report = new_order_rejected(reason, order.as_ref());
                report.exec_id = self.exec_id;
                sender.send(bincode::serialize(&report).unwrap());
            },
            ExecutionTask::NewoOrderMatched(info) => {
                let mut tcr = gen_tcr(info.last_px, info.last_qty, info.order1.as_ref(), &info.order2);
                self.exec_id += 1;
                let mut report = new_order_matched(info.last_px, info.last_qty, info.leaves_qty1, &info.order1);
                report.exec_id = self.exec_id;
                tcr.exec_id = self.exec_id;
                sender.send(bincode::serialize(&report).unwrap());
                let mut report = new_order_matched(info.last_px, info.last_qty, info.leaves_qty2, &info.order2);
                report.exec_id = self.exec_id;
                tcr.counterparty_exec_id = self.exec_id;
                sender.send(bincode::serialize(&report).unwrap());
                sender.send(bincode::serialize(&tcr).unwrap());
            },
            ExecutionTask::CancelRequestAccepted(leaves_qty,cancel_request , order) => {
                self.exec_id += 1;
                let mut report = new_order_cancelled(leaves_qty, cancel_request.as_ref(), order.as_ref());
                report.exec_id = self.exec_id;
                sender.send(bincode::serialize(&report).unwrap());
            },
            ExecutionTask::CancelRequestRejected(reason, cancel_request) => {
                let report = cancel_rejected(reason, cancel_request.as_ref());
                sender.send(bincode::serialize(&report).unwrap());
            }
        }

    }

}

fn new_order_accepted(order : &NewOrder) -> ExecutionReport {
    let mut report = ExecutionReport::new(order);
    report.leaves_qty = order.qty;
    report.exec_type = K_EXEC_TYPE_NEW;
    report.ord_status = K_ORD_STATUS_NEW;
    report
}
fn new_order_rejected(reason : CancelReasonCode, order : &NewOrder) -> ExecutionReport {
    let mut report = ExecutionReport::new(order);
    report.leaves_qty = order.qty;
    report.exec_type = K_EXEC_TYPE_REJECT;
    report.ord_status = K_ORD_STATUS_REJECT;
    report.rejected_reason = reason;
    report
}
fn new_order_matched(last_px : Price, last_qty : Qty, leaves_qty : Qty, order : &NewOrder) -> ExecutionReport {
    let mut report = ExecutionReport::new(order);
    report.exec_type = K_EXEC_TYPE_TRADE;
    if leaves_qty == 0 {
        report.ord_status = K_ORD_STATUS_FILLED;
    }
    else {
        report.ord_status = K_ORD_STATUS_PARTIALLY_FILLED;
    }
    report.cum_qty = order.qty - leaves_qty;
    report.leaves_qty = leaves_qty;
    report.last_px = last_px;
    report.last_qty = last_qty;
    report
}

fn gen_tcr(last_px : Price, last_qty : Qty, order : &NewOrder, couterparty_order : &NewOrder) 
    -> TradeCaptureReport {
    TradeCaptureReport {
        security_id : order.security_id.clone(),
        order_id : order.order_id.clone(),
        pbu_id : order.pbu_id.clone(),
        cl_ord_id : order.cl_ord_id.clone(),
        exec_id : 0,
        counterparty_order_id : couterparty_order.order_id,
        counterparty_pbu_id : couterparty_order.pbu_id,
        counterparty_cl_ord_id : couterparty_order.cl_ord_id,
        counterparty_exec_id : 0,
        last_px : last_px,
        last_qty : last_qty
    }
}

fn new_order_cancelled(leaves_qty : Qty, cancel_request : &CancelRequest, order : &NewOrder) -> ExecutionReport {
    let mut report = ExecutionReport::new(order);
    report.cl_ord_id = cancel_request.cl_ord_id;
    report.orig_cl_ord_id = order.cl_ord_id;
    report.exec_type = K_EXEC_TYPE_CANCELLED;
    report.ord_status = K_ORD_STATUS_CANCELLED;
    report.cum_qty = order.qty - leaves_qty;
    report
}

fn cancel_rejected(reason : CancelReasonCode, cancel_request : &CancelRequest) -> CancelReject {
    CancelReject { 
        order_id: cancel_request.order_id.clone(),
        pbu_id : cancel_request.pbu_id.clone(),
        cl_ord_id : cancel_request.cl_ord_id.clone(),
        orig_cl_ord_id : cancel_request.orig_cl_ord_id.clone(),
        security_id : cancel_request.security_id.clone(),
        rejected_reason : reason
    }
}

#[derive(Serialize)]
struct CancelReject {
    order_id : OrderID,
    pbu_id : PBUID,
    cl_ord_id : ClOrdID,
    orig_cl_ord_id : ClOrdID,
    security_id : SecurityID,
    rejected_reason : CancelReasonCode,
}


#[derive(Serialize)]
struct TradeCaptureReport {
    security_id : SecurityID,
    order_id : OrderID,
    pbu_id : PBUID,
    cl_ord_id : ClOrdID,
    exec_id : ExecID,
    counterparty_order_id : OrderID,
    counterparty_pbu_id : PBUID,
    counterparty_cl_ord_id : ClOrdID,
    counterparty_exec_id : ExecID,
    last_px : Price,
    last_qty : Qty,
}

#[derive(Serialize)]
struct ExecutionReport {
    order_id : OrderID,
    pbu_id : PBUID,
    cl_ord_id : ClOrdID,
    orig_cl_ord_id : ClOrdID,
    security_id : SecurityID,
    side : Side,
    price : Price,
    qty : Qty,
    cum_qty : Qty,
    leaves_qty : Qty,
    rejected_reason : CancelReasonCode,
    exec_type : char,
    ord_status : char,
    last_px : Price,
    last_qty : Qty,
    exec_id : ExecID,
}

impl ExecutionReport {
    pub fn new(order : &NewOrder) -> ExecutionReport{
        ExecutionReport {
            order_id : order.order_id.clone(),
            pbu_id : order.pbu_id.clone(),
            cl_ord_id : order.cl_ord_id.clone(),
            orig_cl_ord_id : to_array(""),
            security_id : order.security_id.clone(),
            side : order.side.clone(),
            price : order.price.clone(),
            qty : order.qty.clone(),
            cum_qty : 0,
            leaves_qty : 0,
            rejected_reason : CancelReasonCode::Passed,
            exec_type : ' ',
            ord_status : ' ' ,
            last_px : 0,
            last_qty : 0,
            exec_id : 0
        }
    }
}