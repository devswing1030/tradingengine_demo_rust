use crate::types::{ Qty, Price, OrderID };

pub trait AuctionOrder {
    fn qty(&self) -> Qty;
    fn price(&self) -> Price;
    fn order_id(&self) -> OrderID;
}

pub struct ConsumedOrder<Order> {
    pub consumed_qty : Qty,
    pub leaves_qty : Qty,
    pub orig_order : std::rc::Rc<Order>
}

impl <Order> ConsumedOrder<Order> {
    pub fn consumed_qty(&self) -> Qty {
        self.consumed_qty
    }
    pub fn leaves_qty(&self) -> Qty {
        self.leaves_qty
    }
    pub fn orig_order(&self) -> std::rc::Rc<Order> {
        self.orig_order.clone()
    }
}

#[cfg(test)]
    /*这里必须加上rc的use，同时下面使用也必须以 std::rc::Rc的形式，否则cargo build不通过，具体原因不清楚 */
    use std::rc::Rc;

    pub struct TestOrder {
        qty : Qty,
        price : Price,
        order_id : OrderID
    }
    impl AuctionOrder for TestOrder {
        fn qty(&self) -> Qty{
            self.qty
        }
        fn price(&self) -> Price{
            self.price
        }
        fn order_id(&self) -> OrderID{
            self.order_id
        }
    }
    impl TestOrder {
        pub fn new(price : Price, qty : Qty, order_id: OrderID) -> std::rc::Rc<TestOrder>
        {
            std::rc::Rc::new(TestOrder {qty, price, order_id})
        }

    }

    pub struct TestOrderGen {
        order_id : OrderID,
    }

    pub type OrderInfo = (Price, Qty, Option<OrderID>);

    impl TestOrderGen {
        pub fn new() -> TestOrderGen {
            TestOrderGen { order_id: 9876543210000 }
        }
        pub fn work(&mut self, orders : &[OrderInfo])-> Vec<std::rc::Rc<TestOrder>> {
            let mut return_orders : Vec<std::rc::Rc<TestOrder>> = Vec::new();

            for order in orders {
                let mut tmp_order_id = self.order_id;
                if let Some(id) = order.2 {
                    tmp_order_id = id;
                }
                else {
                    self.order_id += 1;
                }
                return_orders.push(TestOrder::new(order.0, order.1, tmp_order_id))
            }

            return_orders
        }
    }