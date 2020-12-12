static CMD_AT: &str = "AT\r\n";

use crate::hardware::usart_adapter::UsartAdapter;
use crate::usart::_USART;
pub struct Sim900 {}

impl Sim900 {
    pub fn new() -> Self {
        Sim900 {}
    }

    pub fn init(&self) {
        let (_a, _b) = _USART.get().write_and_wait_answer(CMD_AT.as_bytes());
        let (_a, _b) = _USART.get().read_data();
    }
}
