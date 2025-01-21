use serde::Serialize;

pub type Qty = u64;
pub type Price = i64;
pub type OrderID = u128;
pub type ClOrdID = [u8;10];
pub type PBUID = [u8;6];
pub type SecurityID = [u8;8];
pub type Side = char;
pub type ExecID = u128;

pub const K_BUY: char = 'B';
pub const K_SELL: char = 'S';

pub const K_EXEC_TYPE_NEW : char = '0';
pub const K_EXEC_TYPE_CANCELLED : char = '4';
pub const K_EXEC_TYPE_REJECT: char = '8';
pub const K_EXEC_TYPE_TRADE: char = 'F';

pub const K_ORD_STATUS_NEW : char = '0';
pub const K_ORD_STATUS_PARTIALLY_FILLED : char = '1';
pub const K_ORD_STATUS_FILLED : char = '2';
pub const K_ORD_STATUS_CANCELLED: char = '4';
pub const K_ORD_STATUS_REJECT: char = '8';



#[derive(PartialEq, Eq, Serialize)]
#[derive(Debug)]
pub enum CancelReasonCode {
    Passed = 0,
    Duplicated = 1,
    InvalidSecurity = 2,
    OrderNotExisted = 3,
}

pub fn to_array<const N : usize>(s : &str) -> [u8;N] {
    let mut i = 0;
    let mut result : [u8;N] = [b' ';N];

    let bytes = s.as_bytes();
    
    while i < N && i < bytes.len(){
        result[i] = bytes[i];
        i += 1;
    }
    result
}
pub struct ExeSender {
    pub count : u32,
    pub bytes : usize,
}

impl ExeSender {
    pub fn new() -> ExeSender {
        ExeSender { count: 0, bytes : 0 }
    }
    pub fn send(&mut self, buffer : Vec<u8>) {
        self.count += 1;
        self.bytes += buffer.len();
    }
}