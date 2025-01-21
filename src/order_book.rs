mod price_node;
pub mod auction_order;

use std::collections::{BTreeMap};
use std::collections::btree_map::Iter;
use std::collections::btree_map::IterMut;
use price_node::*;
use auction_order::*;
use std::rc::Rc;
use std::cell::RefCell;

use crate::types::{Qty, Price};

pub struct PriceOrderBook<Order> {
    price_multiplier : i64,
    nodes : BTreeMap<i64, PriceNode<Order>>,
}

impl<Order : AuctionOrder> PriceOrderBook<Order> {
    pub fn create_high_price_priority_order_book() -> PriceOrderBook<Order> {
        PriceOrderBook {
            price_multiplier : -1,
            nodes : BTreeMap::new()
        }
    }

    pub fn create_low_price_priority_order_book() -> PriceOrderBook<Order> {
        PriceOrderBook {
            price_multiplier : 1,
            nodes : BTreeMap::new()
            
        }
    }

    pub fn insert_order(&mut self, order : Rc<Order>) {
        self.insert_order_with_leaves_qty(order.qty(), order);
    }
    
    pub fn insert_order_with_leaves_qty(&mut self, leaves_qty : Qty, order : Rc<Order>) {
        let tmp_price = order.price() * self.price_multiplier;
        match self.nodes.get_mut(&tmp_price) {
            Some(node ) => node.append_order_with_leaves_qty(leaves_qty, order),
            None => {
                let mut node = PriceNode::new();
                let price = order.price();
                node.append_order_with_leaves_qty(leaves_qty, order);
                self.nodes.insert(price * self.price_multiplier, node);
            }
        }

    }

    pub fn remove_order(&mut self, price : i64, order_id : u128) -> Option<ConsumedOrder<Order>> {
        if let Some(node) = self.nodes.get_mut(&(price * self.price_multiplier)) {
            let order = node.remove_order(order_id);
            if node.total() == 0 {
                self.nodes.remove(&(price * self.price_multiplier));
            }
            order
        }
        else {
            None
        }
    }

    pub fn consume_order(&mut self, qty : Qty, limit_price : Price) -> (Qty, Vec<ConsumedOrder<Order>>) {
        let mut orders : Vec<ConsumedOrder<Order>> = Vec::new();

        let mut left_qty = qty;

        loop {
            if let Some(mut entry)= self.nodes.first_entry() {
                if *entry.key() <= limit_price * self.price_multiplier {
                    let node = entry.get_mut();
                    if node.total() < left_qty {
                        left_qty -= node.total();
                        let mut vec = node.consume_order(node.total());
                        orders.append(&mut vec);
                        self.nodes.pop_first();
                    }
                    else {
                        let mut vec = node.consume_order(left_qty);
                        orders.append(&mut vec);
                        left_qty = 0;
                        break;
                    }
                }
                else {
                    break;
                }
            }
            else {
                break;
            }
        }
        (left_qty, orders)

    }

    pub fn price_iter(&self) -> BookPriceIter<'_, Order> {
        BookPriceIter { iter: (self.nodes.iter()), price_multiplier : self.price_multiplier }
    }

    pub fn order_iter_mut(&mut self) -> BookOrderIterMut<'_, Order> {
        BookOrderIterMut::new(self)

    }
}

pub struct BookPriceIter<'a, Order> {
    iter : Iter<'a, i64, PriceNode<Order>>,
    price_multiplier : i64
}

impl <Order : AuctionOrder> BookPriceIter<'_, Order> {
    pub fn next(&mut self) -> Option<(i64, u64)> {
        while let Some(node) = self.iter.next() {
            if node.1.total() != 0 {
                return Some((*(node.0) * self.price_multiplier, node.1.total()))
            }
        }
        None
    }
}

pub struct BookOrderIterMut<'a, Order> {
    price_iter : IterMut<'a, i64, PriceNode<Order>>,
    order_iter : Option<price_node::OrderIterMut<'a, Order>>
}

impl <Order : AuctionOrder> BookOrderIterMut<'_, Order> {
    pub fn new(order_book : &mut PriceOrderBook<Order>) -> BookOrderIterMut<Order>{
        let mut tmp = BookOrderIterMut { price_iter: order_book.nodes.iter_mut() , order_iter : None};

        while let Some(node) = tmp.price_iter.next() {
            if node.1.total() != 0 {
                tmp.order_iter = Some(node.1.order_iter_mut());
                break;
            }
        }
        tmp
    }

    pub fn next(&mut self) -> Option<Rc<RefCell<OrderWithStatus<Order>>>> {

        let has_next = self.order_iter.as_mut().unwrap().has_next();

        if has_next == false {
            self.order_iter = None;
            while let Some(node) = self.price_iter.next() {
                if node.1.total() != 0 {
                    self.order_iter = Some(node.1.order_iter_mut());
                    break;
                }
            }
        }

        if let None = self.order_iter {
            return None
        }

        self.order_iter.as_mut().unwrap().next()

    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use crate::{types::{Price, Qty}};

    #[test]
    fn remove_order() {
        let mut ob = PriceOrderBook::create_low_price_priority_order_book();

        let orders_info = [
            (99, 1, Some(0)),
            (100, 20, None),
            (102, 2, Some(4)),
            (105, 20, None),
            (106, 3, Some(6))
        ];

        let mut gen = TestOrderGen::new();
        gen.work(&orders_info).iter().for_each(|order| ob.insert_order(order.clone()));

        let order = ob.remove_order(99, 0).unwrap();
        assert_eq!(order.consumed_qty(), 1);
        assert_eq!(order.orig_order().price(), 99);
    }

    #[test]
    fn append_order() {
        let mut ob = PriceOrderBook::create_low_price_priority_order_book();

        let orders_info = [
            (99, 1, Some(0)),
            (100, 20, None),
            (101, 20, None),
            (101, 20, None),
            (102, 2, Some(4)),
            (105, 20, None),
            (106, 3, Some(6))
        ];

        let mut gen = TestOrderGen::new();
        gen.work(&orders_info).iter().for_each(|order| ob.insert_order(order.clone()));

        ob.remove_order(99, 0);
        ob.remove_order(102,4);
        ob.remove_order(106,6);

        let mut iter = ob.price_iter();

        assert_eq!(Some((100, 20)), iter.next());
        assert_eq!(Some((101, 40)), iter.next());
        assert_eq!(Some((105, 20)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn traverse_high_price_priority_order_book() {
        let mut ob = PriceOrderBook::create_high_price_priority_order_book();
        let orders_info = [
            (99, 1, Some(0)),
            (100, 20, None),
            (101, 20, None),
            (101, 20, None),
            (102, 2, Some(4)),
            (105, 20, None),
            (106, 3, Some(6))
        ];
        let mut gen = TestOrderGen::new();
        gen.work(&orders_info).iter().for_each(|order| ob.insert_order(order.clone()));

        ob.remove_order(99,0);
        ob.remove_order(102,4);
        ob.remove_order(106,6);

        let mut iter = ob.price_iter();

        assert_eq!(Some((105, 20)), iter.next());
        assert_eq!(Some((101, 40)), iter.next());
        assert_eq!(Some((100, 20)), iter.next());
        assert_eq!(None, iter.next());
    }

    fn assert_order(order : Rc<RefCell<OrderWithStatus<TestOrder>>>, price : Price, qty : Qty) {
        assert_eq!(order.borrow().orig_order().price(), price);
        assert_eq!(order.borrow().leaves_qty(), qty);
    }

    #[test]
    fn traverse_order() {
        let mut ob = PriceOrderBook::create_low_price_priority_order_book();

        let orders_info = [
            (100, 20, None),
            (100, 1, Some(1)),
            (100, 30, None),
            (101, 20, None),
            (101, 50, None),
            (102, 2, Some(5)),
            (105, 20, None)
        ];

        let mut gen = TestOrderGen::new();
        gen.work(&orders_info).iter().for_each(|order| ob.insert_order(order.clone()));

        ob.remove_order(100, 1);
        ob.remove_order(102, 5);

        let mut iter = ob.order_iter_mut();

        let order = iter.next().unwrap();
        assert_order(order, 100, 20);

        let order = iter.next().unwrap();
        assert_order(order, 100, 30);

        let order = iter.next().unwrap();
        assert_order(order, 101, 20);

        let order = iter.next().unwrap();
        assert_order(order, 101, 50);

        let order = iter.next().unwrap();
        assert_order(order, 105, 20);

        if let None = iter.next() {
        }
        else {
            assert!(false);
        }

    }
    #[test]
    fn consume_order() {
        let mut ob = PriceOrderBook::create_low_price_priority_order_book();
        let orders_info = [
            (100, 10, None),
            (100, 10, None),
            (101, 10, None),
            (101, 10, None),
            (102, 10, None),
            (105, 10, None),
            (106, 10, None),
            (107, 10, None),
            (108, 10, None),
            (108, 20, None),
        ];
        let mut gen = TestOrderGen::new();
        gen.work(&orders_info).iter().for_each(|order| ob.insert_order(order.clone()));

        let (leaves_qty, consumed) = ob.consume_order(100, 101);
        assert_eq!(consumed.len(), 4);
        assert_eq!(leaves_qty, 60);
        assert_eq!(consumed[0].orig_order().price(), 100);
        assert_eq!(consumed[0].consumed_qty(), 10);
        assert_eq!(consumed[1].orig_order().price(), 100);
        assert_eq!(consumed[1].consumed_qty(), 10);
        assert_eq!(consumed[2].orig_order().price(), 101);
        assert_eq!(consumed[2].consumed_qty(), 10);
        assert_eq!(consumed[3].orig_order().price(), 101);
        assert_eq!(consumed[3].consumed_qty(), 10);

        let (leaves_qty, consumed) = ob.consume_order(13, 200);
        assert_eq!(consumed.len(), 2);
        assert_eq!(leaves_qty, 0);
        assert_eq!(consumed[0].orig_order().price(), 102);
        assert_eq!(consumed[0].consumed_qty(), 10);
        assert_eq!(consumed[1].orig_order().price(), 105);
        assert_eq!(consumed[1].consumed_qty(), 3);

        let (leaves_qty, consumed)= ob.consume_order(17, 200);
        assert_eq!(consumed.len(), 2);
        assert_eq!(leaves_qty, 0);
        assert_eq!(consumed[0].orig_order().price(), 105);
        assert_eq!(consumed[0].consumed_qty(), 7);
        assert_eq!(consumed[1].orig_order().price(), 106);
        assert_eq!(consumed[1].consumed_qty(), 10);

        let (leaves_qty, consumed) = ob.consume_order(100, 200);
        assert_eq!(consumed.len(), 3);
        assert_eq!(leaves_qty, 60);
        assert_eq!(consumed[0].orig_order().price(), 107);
        assert_eq!(consumed[0].consumed_qty(), 10);
        assert_eq!(consumed[1].orig_order().price(), 108);
        assert_eq!(consumed[1].consumed_qty(), 10);
        assert_eq!(consumed[2].orig_order().price(), 108);
        assert_eq!(consumed[2].consumed_qty(), 20);

        let (_, consumed) = ob.consume_order(100, 200);
        assert_eq!(consumed.len(), 0);

    }
}
