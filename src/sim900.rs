#[allow(dead_code)]
static SIM900_AT: &str = "AT\r\n";
static SIM900_DATA_MODE: &str = "AT+CBST=71,0,1\r\n";
//static SIM900_INSERT_PINCODE: &str = "AT+CPIN=\"";
static SIM900_GET_SIM_STATUS: &str = "AT+CPIN?\r\n";
//static SIM900_GET_MONEY: &str = "ATD#100#;\r\n";
//static SIM900_GET_OPSOS: &str = "AT+COPS?\r\n";
static SIM900_TEXT_MODE_ON: &str = "AT+CMGF=1\r\n";
//static SIM900_AON_ENABLE: &str = "AT+CLIP=1\r\n";
//static SIM900_ECHO_OFF: &str = "ATE0\r\n";
static SIM900_END: &str = "\r\n";
//static SIM900_RING: &str = "ATD";
//static SIM900_ATA: &str = "ATA\r\n";
//static SIM900_LEAVE_CMD_MODE: &str = "+++";
//static SIM900_DISCONNECT: &str = "ATH0\r\n";
//static SIM900_GET_TIME: &str = "AT+CCLK?\r\n";
//static SIM900_SET_TIME: &str = "AT+CCLK=\"";
static SIM900_SEND_SMS: &str = "AT+CMGS=\"";
//static SIM900_TERMINATOR: &str = ";";
static SIM900_CMD_ENTER: &str = "\x1a\r";
//static SIM900_CMD_CANCEL: &str = "\x1b\r";
//static ANSWER_EMPTY_LINE: &str = "\r\n";
//static ANSWER_OPSOS: &str = "\r\n+COPS";
//static ANSWER_MONEY: &str = "\r\n+CUSD:";
//static ANSWER_SIM_STATUS: &str = "\r\n+CPIN:";
//static ANSWER_INCOMING_RING: &str = "\r\nRING\r\n";
//static ANSWER_INCOMING_PHONE_NUMBER: &str = "\r\n+CLIP:";
//static ANSWER_ENTER_SMS: &str = ">";
static ANSWER_OK: &str = "\r\nOK";
static ANSWER_ERROR: &str = "\r\nERROR";
//static ANSWER_CONNECT: &str = "\r\nCONNECT";
//static ANSWER_NO_DIALTONE: &str = "\r\nNO DIALTONE";
//static ANSWER_NO_CARRIER: &str = "\r\nNO CARRIER";

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
fn write_and_wait_answer<T: TimeType>(arr: &str, timeout: T) -> Option<Span> {
    let mut t = Timer::new();
    _USART.get().prepare_to_read();
    _USART.get().write_data(arr.as_bytes());
    t.reset();
    while t.waiting(&timeout) {
        if let Some(r) = _USART.get().read_result() {
            return Some(r);
        }
    }
    return None;
}

pub enum RequestError {
    ETimeout,
    ENoDevice,
    ENoAnswer,
    EAnswerError,
    EAnswerUnknown,
    EBadRequest,
}

fn parse(s: &str, len: usize) -> Result<(), RequestError> {
    let pos = s.find('\n');
    let result = match pos {
        None => return Err(RequestError::ENoDevice),
        Some(x) if x == len - 1 => return Err(RequestError::ENoAnswer),
        Some(x) => {
            if let Some(_) = s[x..].find(ANSWER_OK) {
                Ok(())
            } else if let Some(_) = s[x..].find(ANSWER_ERROR) {
                Err(RequestError::EAnswerError)
            } else {
                Err(RequestError::EAnswerUnknown)
            }
        }
    };
    result
}

///send data and blocking waiting result
fn request<T: TimeType>(arr: &str, timeout: T) -> Result<(), RequestError> {
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
        let res = request(SIM900_AT, Sim900::TIMEOUT);
        if res.is_err() {
            self.pin.set_high().unwrap();
            self.t.wait(2.sec());
            self.pin.set_low().unwrap();
            self.t.wait(1.sec());
        }
        Sim900Powered {
            state:Sim900State::Unknown
            //pin: self.pin,
            //t: self.t,
        }
    }

    pub fn init(&mut self) {
        self.pin.set_low().unwrap();
    }
}

pub enum Sim900State {
    Unknown,
    Good,
    NoAnswer,
    BadAnswer,
    NoSim,
}
pub struct Sim900Powered {
    state: Sim900State,
}

extern crate heapless;
use heapless::consts::*;
use heapless::String;
impl Sim900Powered {
    pub fn get_state(self) -> Sim900State {
        return self.state;
    }
    pub fn setup(&mut self) -> Result<(), RequestError> {
        self.handle_request(|| -> Result<(), RequestError> {
            request(SIM900_TEXT_MODE_ON, Sim900::TIMEOUT)?;
            request(SIM900_DATA_MODE, Sim900::TIMEOUT) //set 9600 bod in data mode
        }())?;
        let res = request(SIM900_GET_SIM_STATUS, Sim900::TIMEOUT); //check sim
        if let Err(RequestError::EAnswerError) = res {
            self.state = Sim900State::NoSim;
            res
        } else {
            self.handle_request(res)
        }
    }

    fn handle_request(&mut self, x: Result<(), RequestError>) -> Result<(), RequestError> {
        match x {
            Ok(_) => self.state = Sim900State::Good,
            Err(RequestError::EBadRequest) => self.state = Sim900State::Good,
            Err(RequestError::EAnswerError) | Err(RequestError::EAnswerUnknown) => {
                self.state = Sim900State::BadAnswer;
            }
            Err(RequestError::ENoDevice)
            | Err(RequestError::ENoAnswer)
            | Err(RequestError::ETimeout) => {
                self.state = Sim900State::NoAnswer;
            }
        };
        x
    }

    pub fn send_sms(&mut self, telephone: &str, msg: &str) -> Result<(), RequestError> {
        self.handle_request(|| -> Result<(), RequestError> {
            //24?
            let mut cmd: String<U50> = String::from(SIM900_SEND_SMS);
            cmd.push_str(telephone)?;
            cmd.push('\"')?;
            cmd.push_str(SIM900_END)?;
            request(&cmd, Sim900::TIMEOUT)?;
            let mut message: String<U50> = String::from(msg);
            message.push_str(SIM900_CMD_ENTER)?;
            request(&message, Sim900::TIMEOUT)
        }())
    }
}

impl From<()> for RequestError {
    fn from(_: ()) -> Self {
        RequestError::EBadRequest
    }
}
