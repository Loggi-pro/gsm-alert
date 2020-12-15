static CMD_AT: &str = "AT\r\n";

use crate::hal::gpio::{Output, PushPull, Pxx};
use crate::timer::{CounterTypeExt, Timer};
use crate::usart::_USART;
use embedded_hal::digital::v2::OutputPin;
pub struct Sim900 {
    pin: Pxx<Output<PushPull>>,
    t: Timer,
}
impl Sim900 {
    pub fn new(pin: Pxx<Output<PushPull>>) -> Self {
        Sim900 {
            pin,
            t: Timer::new(),
        }
    }
    pub fn power_on(mut self) -> Sim900Powered {
        self.pin.set_high().unwrap();
        self.t.wait(2.sec());
        self.pin.set_low().unwrap();
        Sim900Powered {
            //pin: self.pin,
            //t: self.t,
        }
    }

    pub fn init(&mut self) {
        self.pin.set_low().unwrap();
        //let (_a, _b) = _USART.get().write_and_wait_answer(CMD_AT.as_bytes());
        //let (_a, _b) = _USART.get().read_data();
    }
}

pub struct Sim900Powered {
    //pin: Pxx<Output<PushPull>>,
// t: Timer,
}

impl Sim900Powered {
    pub fn setup(&self) {
        //let (_a, _b) = _USART.get().write_and_wait_answer(CMD_AT.as_bytes());
        //let (_a, _b) = _USART.get().read_data();
    }
}
