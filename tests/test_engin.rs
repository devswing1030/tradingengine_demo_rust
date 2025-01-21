use trading::engin::Engin;
use trading::messages::CancelRequest;
use trading::messages::NewOrder;
use trading::messages::PreProcessorTask;
use trading::types::*;

use std::time::Instant;
use rand::Rng;

pub struct RandomOrderGen {
    order_id : OrderID,
    cl_order_id : u64,
    rng : rand::rngs::ThreadRng,
}
impl RandomOrderGen {
    fn new() -> RandomOrderGen {
        let rng = rand::thread_rng();
        RandomOrderGen { order_id: 0, cl_order_id: 0, rng : rng  }
    }
    fn gen_order(&mut self) -> Box<NewOrder> {
        let rand_price : u16 = self.rng.gen_range(1..=200);
        let rand_qty : u16 = self.rng.gen_range(1..=10000);

        self.order_id += 1;
        self.cl_order_id += 1;
        Box::new(NewOrder { 
            order_id: self.order_id,
            pbu_id: to_array(&rand_qty.to_string()),
            cl_ord_id: to_array(&format!("{:X}{:X}", self.cl_order_id, rand_price)),
            security_id: to_array("SEC001"), 
            side: {if rand_price % 2 == 1 {K_BUY} else {K_SELL}}, 
            price: From::from(rand_price), 
            qty: From::from(rand_qty),
        })
    }
    pub fn get_cancel_request(&mut self, orig_order : &Box<NewOrder>) -> Box<CancelRequest> {
        self.order_id += 1;
        self.cl_order_id += 1;
        Box::new(CancelRequest { order_id: self.order_id, 
            pbu_id: orig_order.pbu_id.clone(), 
            cl_ord_id: to_array(&self.order_id.to_string()),
            orig_cl_ord_id: orig_order.cl_ord_id.clone(), 
            security_id: orig_order.security_id.clone() })
    }
}

#[test]
fn test_order_process() {
    let mut gen = trading::messages::OrderGen::new();
    let mut sender = ExeSender::new();

    let mut engin = Engin::new(sender);

    let order = gen.gen_order(K_BUY, 100, 30);
    let cancel_request1 = gen.get_cancel_request(&order);
    engin.process(PreProcessorTask::NewOrder(order));
    let order = gen.gen_order(K_BUY, 110, 50);
    engin.process(PreProcessorTask::NewOrder(order));
    let order = gen.gen_order(K_SELL, 100, 120);
    let cancel_request2 = gen.get_cancel_request(&order);
    engin.process(PreProcessorTask::NewOrder(order));

    engin.process(PreProcessorTask::CancelRequest(cancel_request1));
    engin.process(PreProcessorTask::CancelRequest(cancel_request2));

    let exe = engin.exe_rx.recv().unwrap().unwrap();
    match exe {
        trading::messages::ExecutionTask::NewOrderAccepted(order) => {
            assert_eq!(order.qty, 30);
        },
        _ => assert!(false),
    }
    let exe = engin.exe_rx.recv().unwrap().unwrap();
    match exe {
        trading::messages::ExecutionTask::NewOrderAccepted(order) => {
            assert_eq!(order.qty, 50);
        },
        _ => { dbg!(exe); assert!(false); },
    }
    let exe = engin.exe_rx.recv().unwrap().unwrap();
    match exe {
        trading::messages::ExecutionTask::NewOrderAccepted(order) => {
            assert_eq!(order.qty, 120);
        },
        _ => { dbg!(exe); assert!(false); },
    }
    let exe = engin.exe_rx.recv().unwrap().unwrap();
    match exe {
        trading::messages::ExecutionTask::NewoOrderMatched(info) => {
            assert_eq!(info.last_px, 110);
            assert_eq!(info.last_qty, 50);
        },
        _ => { dbg!(exe); assert!(false); },
    }
    let exe = engin.exe_rx.recv().unwrap().unwrap();
    match exe {
        trading::messages::ExecutionTask::NewoOrderMatched(info) => {
            assert_eq!(info.last_px, 100);
            assert_eq!(info.last_qty, 30);
            assert_eq!(info.leaves_qty1, 40);
        },
        _ => { dbg!(exe); assert!(false); },
    }
    let exe = engin.exe_rx.recv().unwrap().unwrap();
    match exe {
        trading::messages::ExecutionTask::CancelRequestRejected(code, _) => {
            assert_eq!(code, CancelReasonCode::OrderNotExisted);
        },
        _ => { dbg!(exe); assert!(false); },
    }
    let exe = engin.exe_rx.recv().unwrap().unwrap();
    match exe {
        trading::messages::ExecutionTask::CancelRequestAccepted(qty,_ ,_) => {
            assert_eq!(qty, 40);
        },
        _ => { dbg!(exe); assert!(false); },
    }

    engin.close();
}


#[test]
fn test_engin() {
    let mut gen = RandomOrderGen::new();
    let mut orders = Vec::new();

    let mut rng = rand::thread_rng();

    let mut count = 50000000;
    while count > 0 {
        let order = gen.gen_order();
        let cancel_request = gen.get_cancel_request(&order);
        orders.push(PreProcessorTask::NewOrder(order));
        count -= 1;
        let rand_num = rng.gen_range(1..=100);
        if rand_num % 100 < 30 {
            orders.push(PreProcessorTask::CancelRequest(cancel_request));
            count -= 1;
        }

    }

    let mut sender = ExeSender::new();

    let mut engin = Engin::new(sender);

    let now = Instant::now();

    orders.into_iter().for_each(|task| {
        //println!("price {}, qty {}", order.price, order.qty);
        engin.engin_tx.send(Some(task)).unwrap();
    });
    engin.engin_tx.send(None).unwrap();

    sender = engin.close();

    let elapsed_time = now.elapsed();

    println!("generate {} execution reports", sender.count);
    println!("send {} bytes", sender.bytes);
    println!("running {} seconds", elapsed_time.as_secs());

}


