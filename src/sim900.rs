static CMD_AT: &str = "AT\r\n";

use crate::hal::gpio::{Output, PushPull, Pxx};
use crate::timer::{CounterTypeExt, TimeType, Timer};
use crate::usart::_USART;
use crate::utils::span::Span;
use embedded_hal::digital::v2::OutputPin;
pub struct Sim900 {
    pin: Pxx<Output<PushPull>>,
    t: Timer,
}

///send data and blocking waiting result
fn write_and_wait_answer<T: TimeType>(arr: &[u8], timeout: T) -> Option<Span> {
    let mut t = Timer::new();
    _USART.get().prepare_to_read();
    _USART.get().write_data(arr);
    t.reset();
    while t.waiting(&timeout) {
        if let Some(r) = _USART.get().read_result() {
            return Some(r);
        }
    }
    return None;
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
        let o = write_and_wait_answer(CMD_AT.as_bytes(), 0.mil());
        let mut a = 0;
        if let Some(Span(arr, l)) = o {
            a = l; //dont go here
        }
        let o = write_and_wait_answer(CMD_AT.as_bytes(), 100.mil());
        let mut b = 0;
        if let Some(Span(arr, l)) = o {
            b = l; //go here
        }
        b = b + 1 - 1;
    }
}
