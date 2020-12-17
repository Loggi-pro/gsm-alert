static CMD_AT: &str = "AT\r\n";

use crate::hal::gpio::{Output, PushPull, Pxx};
use crate::timer::{CounterTypeExt, MilliSeconds, TimeType, Timer};
use crate::usart::_USART;
use crate::utils::span::Span;
use core::str;
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

enum RequestError {
    ETimeout,
    ENoDevice,
    ENoAnswer,
    EAnswerError,
}

fn parse(s: &str, len: usize) -> Result<(), RequestError> {
    let pos = s.find('\n');
    let result = match pos {
        None => return Err(RequestError::ENoDevice),
        Some(x) if x == len - 1 => return Err(RequestError::ENoAnswer),
        Some(x) => {
            if let Some(_) = s[x..].find("OK") {
                Ok(())
            } else {
                Err(RequestError::EAnswerError)
            }
        }
    };
    result
}

///send data and blocking waiting result
fn request<T: TimeType>(arr: &[u8], timeout: T) -> Result<(), RequestError> {
    let res = write_and_wait_answer(arr, timeout);
    let result = match res {
        None => Err(RequestError::ETimeout),
        Some(Span(arr, len)) => parse(str::from_utf8(&arr[0..len]).unwrap(), len),
    };
    return result;
}

impl Sim900 {
    const TIMEOUT: MilliSeconds = MilliSeconds(300_u16);
    pub fn new(pin: Pxx<Output<PushPull>>) -> Self {
        Sim900 {
            pin,
            t: Timer::new(),
        }
    }
    pub fn power_on(mut self) -> Sim900Powered {
        //try AT if device already powered
        let res = request(CMD_AT.as_bytes(), Sim900::TIMEOUT);
        if res.is_err() {
            self.pin.set_high().unwrap();
            self.t.wait(2.sec());
            self.pin.set_low().unwrap();
            self.t.wait(1.sec());
        }
        Sim900Powered {
            //pin: self.pin,
            //t: self.t,
        }
    }

    pub fn init(&mut self) {
        self.pin.set_low().unwrap();
    }
}

pub struct Sim900Powered {}

impl Sim900Powered {
    pub fn setup(&self) {
        let res = request(CMD_AT.as_bytes(), Sim900::TIMEOUT);

        let mut a = 0;
        match res {
            Ok(_) => a = 1,
            Err(RequestError::ENoDevice) => a = 2,
            Err(RequestError::ENoAnswer) => a = 3,
            Err(RequestError::EAnswerError) => a = 4,
            Err(RequestError::ETimeout) => a = 5,
        }

        a = a + 1 - 1;
    }
}
