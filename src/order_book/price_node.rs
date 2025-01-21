
use std::collections::{LinkedList, linked_list::IterMut};
use std::collections::BTreeMap;
use std::cell::RefCell;
use std::rc::Rc;
use crate::types::{Qty, OrderID};
use crate::order_book::AuctionOrder;
use crate::order_book::ConsumedOrder;

pub struct OrderWithStatus<Order> {
    leaves_qty : Qty,
    orig_order : Rc<Order>
}

impl <Order> OrderWithStatus<Order> {
    pub fn leaves_qty(&self) -> Qty {
        self.leaves_qty
    }
    pub fn orig_order(&self) -> Rc<Order> {
        self.orig_order.clone()
    }
}

pub struct PriceNode<Order> {
    total : Qty,
    zero_orders : usize,
    order_list : LinkedList<Rc<RefCell<OrderWithStatus<Order>>>>,
    order_map : BTreeMap<u128, Rc<RefCell<OrderWithStatus<Order>>>>
}

impl<Order : AuctionOrder> PriceNode<Order> {
    pub fn new() -> PriceNode<Order> {
        PriceNode { total: 0, zero_orders: 0, order_list: LinkedList::new(), order_map : BTreeMap::new() }
    }
    pub fn append_order(&mut self, order : Rc<Order>) {
        self.append_order_with_leaves_qty(order.qty(), order);
    }
    pub fn append_order_with_leaves_qty(&mut self, leaves_qty : Qty, order : Rc<Order>) {
        if leaves_qty == 0 {
            panic!("Order with leaves qty of 0 should not be appended!");
        }

        self.total += leaves_qty;

        let tmp_order = Rc::new(RefCell::new(OrderWithStatus{ leaves_qty : leaves_qty, orig_order : order.clone()}));
        self.order_list.push_back(tmp_order.clone());
        self.order_map.insert(order.order_id(), tmp_order.clone());

    }
    pub fn remove_order(&mut self, order_id : OrderID) -> Option<ConsumedOrder<Order>>
    {
        let order = self.order_map.remove(&order_id);
        if let Some(tmp_order) = order {
            let qty = tmp_order.borrow().leaves_qty;
            tmp_order.borrow_mut().leaves_qty = 0;
            self.total -= qty;
            self.zero_orders += 1;

            Some(ConsumedOrder {consumed_qty : qty, leaves_qty : 0, orig_order : tmp_order.borrow().orig_order()})
        }
        else {
            None
        }
    }
    pub fn consume_order(&mut self, qty : Qty) -> Vec<ConsumedOrder<Order>> {
        let mut orders = Vec::new();
        let mut left_qty = qty;
        loop {
            if let Some(order) = self.order_list.front() {
                if order.borrow().leaves_qty() == 0 {
                    self.order_map.remove(&order.borrow().orig_order().order_id());
                    self.order_list.pop_front();
                    continue;
                }
                if left_qty > order.borrow().leaves_qty() {
                    orders.push(ConsumedOrder { consumed_qty: order.borrow().leaves_qty(), leaves_qty : 0, orig_order: order.borrow().orig_order().clone() });
                    left_qty -= order.borrow().leaves_qty();
                    self.total -= order.borrow().leaves_qty();
                    self.order_map.remove(&order.borrow().orig_order().order_id());
                    self.order_list.pop_front();
                    
                }
                else {
                    order.borrow_mut().leaves_qty -= left_qty;
                    orders.push(ConsumedOrder { consumed_qty: left_qty, leaves_qty : order.borrow().leaves_qty, orig_order: order.borrow().orig_order().clone() });
                    self.total -= left_qty;
                    break;
                }
            }
            else {
                break;
            }
        }

        orders

    }
    pub fn total(&self) -> Qty{
        self.total
    }
    fn remove_front_and_tail_zero_order(&mut self) {
        loop {
            match self.order_list.front() {
                Some(order) =>
                {
                    if order.borrow().leaves_qty == 0 {
                        self.order_map.remove(&order.borrow().orig_order().order_id());
                        self.order_list.pop_front();
                        self.zero_orders -= 1;
                    }
                    else {
                        break;
                    }
                },
                None => ()
            }
        }
        loop {
            match self.order_list.back() {
                Some(order) =>
                {
                    if order.borrow().leaves_qty == 0 {
                        self.order_map.remove(&order.borrow().orig_order().order_id());
                        self.order_list.pop_back();
                        self.zero_orders -= 1;
                    }
                    else {
                        break;
                    }
                },
                None => ()
            }
        }
    }
    pub fn order_iter_mut(&mut self) -> OrderIterMut<'_, Order> {
        OrderIterMut::new(self)
    }
}

pub struct OrderIterMut<'a, Order> {
    iter : IterMut<'a, Rc<RefCell<OrderWithStatus<Order>>>>,
    len : usize,   //list 中的元素个数
    curr : usize  // 当前遍历到第几个
}

impl <Order : AuctionOrder> OrderIterMut<'_, Order> {
    pub fn new(node : &mut PriceNode<Order>) -> OrderIterMut<Order> {
        let tmp_len = node.order_list.len();
        node.remove_front_and_tail_zero_order();
        OrderIterMut {iter : node.order_list.iter_mut(), len : tmp_len, curr : 0 }
    }
    pub fn next(&mut self) -> Option<Rc<RefCell<OrderWithStatus<Order>>>> {
        while let Some(order) = self.iter.next() {
            self.curr += 1;
            if order.borrow().leaves_qty != 0 {
                return Some(order.clone())
            }
        }
        None
    }
    pub fn has_next(&self) -> bool {
        // 因为在创建本实例时会清除首尾的数量为0道委托，所以下面的条件成立则表示未遍历完
        self.curr < self.len
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::order_book::auction_order::*;
    
    #[test]
    fn append_order() {
        let order_id : u128 = 0;
        let mut node : PriceNode<TestOrder> = PriceNode::new();
        let order = TestOrder::new(50, 100, order_id);
        node.append_order(order);
        assert_eq!(node.total(), 100);
    }
    #[test]
    fn skip_zero_order() {

        let orders_info   = [
            (10, 81, Some(0)),
            (10, 91, Some(1)),
            (10, 100, None),
            (10, 81, Some(3)),
            (10, 91, Some(4)),
            (10, 200, None),
            (10, 81, Some(6))
        ];
        let mut gen = TestOrderGen::new();
        let orders = gen.work(&orders_info);
        let mut node : PriceNode<TestOrder> = PriceNode::new();
        for order in orders {
            node.append_order(order);
        }
        node.remove_order(0);
        node.remove_order(1);
        node.remove_order(3);
        node.remove_order(4);
        node.remove_order(6);

        let mut iter = node.order_iter_mut();

        assert_eq!(iter.next().unwrap().borrow().leaves_qty, 100);
        assert_eq!(iter.next().unwrap().borrow().leaves_qty, 200);

        if let Some(_order) = iter.next() {
            assert!(false);
        }
        assert_eq!(node.total(), 300);

    }
    #[test]
    fn consume_order() {
        let orders_info   = [
            (10, 100, None),
            (10, 200, None),
            (10, 300, None),
            (10, 400, None),
            (10, 500, None),
        ];
        let mut gen = TestOrderGen::new();
        let orders = gen.work(&orders_info);
        let mut node : PriceNode<TestOrder> = PriceNode::new();
        for order in orders {
            node.append_order(order);
        }

        let consumed = node.consume_order(150);
        assert_eq!(consumed.len(), 2);
        assert_eq!(node.total(), 1350);
        assert_eq!(consumed[0].consumed_qty(), 100);
        assert_eq!(consumed[0].leaves_qty(), 0);
        assert_eq!(consumed[1].consumed_qty(), 50);
        assert_eq!(consumed[1].leaves_qty(), 150);

        let consumed = node.consume_order(450);
        assert_eq!(consumed.len(), 2);
        assert_eq!(node.total(), 900);
        assert_eq!(consumed[0].consumed_qty(), 150);
        assert_eq!(consumed[1].consumed_qty(), 300);

        let consumed = node.consume_order(1000);
        assert_eq!(consumed.len(), 2);
        assert_eq!(node.total(), 0);
        assert_eq!(consumed[0].consumed_qty(), 400);
        assert_eq!(consumed[1].consumed_qty(), 500);

        let consumed = node.consume_order(1000);
        assert_eq!(consumed.len(), 0);
    }
}