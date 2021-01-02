#[allow(dead_code)]
static SIM900_AT: &str = "AT\r\n";
static SIM900_DATA_MODE: &str = "AT+CBST=71,0,1\r\n";
//static SIM900_INSERT_PINCODE: &str = "AT+CPIN=\"";
static SIM900_GET_SIM_STATUS: &str = "AT+CPIN?\r\n";
//static SIM900_GET_MONEY: &str = "ATD#100#;\r\n";
//static SIM900_GET_OPSOS: &str = "AT+COPS?\r\n";
static SIM900_TEXT_MODE_ON: &str = "AT+CMGF=1\r\n";
static SIM900_PDU_MODE_ON: &str = "AT+CMGF=0\r\n";
//static SIM900_UTF_MODE: &str = "AT+CSCS=\"UCS2\"\r\n";
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

extern crate heapless;
use core::fmt::Write;
use heapless::consts::*;
use heapless::String;
///send data and blocking waiting result
fn write_and_wait_answer<'a, T: TimeType>(arr: &str, timeout: T) -> Option<Span<'a>> {
    //let mut t = Timer::new();
    _USART.get().prepare_to_read();
    _USART.get().write_data(arr.as_bytes());
    _USART.get().read_timeout(timeout)
    //if let Some(r) = _USART.get().read_timeout(timeout) {
    //   return r;
    //}
    // return None;
}

pub enum RequestError<'a> {
    ETimeout,
    ENoDevice,
    ENoAnswer,
    EAnswerError,
    EAnswerUnknown(&'a str),
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
                Err(RequestError::EAnswerUnknown(s))
            }
        }
    };
    result
}

///send data and blocking waiting result
fn request<'a, T: TimeType>(arr: &str, timeout: T) -> Result<(), RequestError<'a>> {
    let res = write_and_wait_answer(arr, timeout);
    let result = match res {
        None => Err(RequestError::ETimeout),
        Some(Span(data, len)) => {
            let s = str::from_utf8(&data[0..len]);
            if s.is_ok() {
                parse(s.unwrap(), len)
            } else {
                Err(RequestError::ENoAnswer)
            }
        }
    };
    return result;
}

#[derive(Copy, Clone)]
pub enum Sim900State {
    Unknown,
    Good,
    NoAnswer,
    BadAnswer,
    NoSim,
}
pub struct Sim900 {
    state: Sim900State,
    pin: Pxx<Output<PushPull>>,
    timer: Timer,
}

fn expect_str<'a>(r: Result<(), RequestError<'a>>, s: &str) -> Result<(), RequestError<'a>> {
    return match r {
        Err(RequestError::EAnswerUnknown(data)) => {
            if data.find(s).is_some() {
                Ok(())
            } else {
                r
            }
        }
        _ => r,
    };
}

impl Sim900 {
    const TIMEOUT: MilliSeconds = MilliSeconds(300_u16);

    pub fn new(mut pin: Pxx<Output<PushPull>>) -> Self {
        pin.set_low().unwrap();
        Sim900 {
            state: Sim900State::Unknown,
            pin: pin,
            timer: Timer::new(),
        }
    }
    pub fn toggle_power(&mut self) {
        self.pin.set_high().unwrap();
        self.timer.wait(2.sec());
        self.pin.set_low().unwrap();
    }
    pub fn get_state(&self) -> Sim900State {
        return self.state;
    }
    pub fn setup(&mut self) -> Result<(), RequestError> {
        self.handle_request(|| -> Result<(), RequestError> {
            request(SIM900_PDU_MODE_ON, Sim900::TIMEOUT)?; //pdu mode for cyrillic
            request(SIM900_DATA_MODE, Sim900::TIMEOUT) //set 9600 bod in data mode
                                                       //request(SIM900_UTF_MODE, Sim900::TIMEOUT) //set Unicode for sms
        }())?;
        let res = request(SIM900_GET_SIM_STATUS, 1.sec()); //check sim
        if let Err(RequestError::EAnswerError) = res {
            self.state = Sim900State::NoSim;
            res
        } else {
            self.handle_request(res)
        }
    }
    pub fn is_online<'a>(&mut self) -> Result<(), RequestError<'a>> {
        self.handle_request(request(SIM900_AT, Sim900::TIMEOUT))
    }
    pub fn power_on<'a>(&mut self) -> Result<(), RequestError<'a>> {
        //try AT if device already powered
        let res = self.is_online();
        if res.is_ok() {
            return res;
        }
        self.toggle_power();
        self.timer.wait(2.sec());
        let res = self.is_online();
        res
    }
    pub fn power_off(&mut self) {
        let res = request(SIM900_AT, Sim900::TIMEOUT);
        if res.is_ok() {
            self.toggle_power();
        }
    }

    fn handle_request<'a>(
        &mut self,
        x: Result<(), RequestError<'a>>,
    ) -> Result<(), RequestError<'a>> {
        match x {
            Ok(_) => self.state = Sim900State::Good,
            Err(RequestError::EBadRequest) => self.state = Sim900State::Good,
            Err(RequestError::EAnswerError) => self.state = Sim900State::BadAnswer,
            Err(RequestError::EAnswerUnknown(_)) => self.state = Sim900State::BadAnswer,
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
            let _ = request(&cmd, Sim900::TIMEOUT); //module not answer, ignore
            let mut message: String<U100> = String::from(msg);
            message.push_str(SIM900_CMD_ENTER)?;
            request(&message, Sim900::TIMEOUT)
        }())
    }
    pub fn send_pdu_sms(&mut self, msg: &str) -> bool {
        if let Sim900State::Good = self.state {
            self.handle_request(|| -> Result<(), RequestError> {
                let mut cmd: String<U50> = String::from("AT+CMGS=");
                let len = (msg.len() - 2) / 2;
                write!(cmd, "{}\r", len)?;
                expect_str(request(&cmd, Sim900::TIMEOUT), ">")?;
                let mut message: String<U200> = String::from(msg);
                message.push_str(SIM900_CMD_ENTER)?;
                let _ = request(&message, Sim900::TIMEOUT); //ignore answer
                Ok(())
            }())
            .is_ok()
        } else {
            false
        }
    }
}

impl<'a> From<()> for RequestError<'_> {
    fn from(_: ()) -> Self {
        RequestError::EBadRequest
    }
}

impl<'a> From<core::fmt::Error> for RequestError<'_> {
    fn from(_: core::fmt::Error) -> Self {
        RequestError::EBadRequest
    }
}
