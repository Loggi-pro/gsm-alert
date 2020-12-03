#![no_std]
#![no_main]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust#53964
extern crate panic_halt; // panic hnadler
use cortex_m_rt::entry;
extern crate hal;
use crate::hal::{
    gpio::{gpioc, Output, PushPull},
    pac::{interrupt, Peripherals},
    prelude::*,
    serial::{self, Serial},
};
use core::cell::RefCell;
use core::sync::atomic::{self, Ordering};

#[allow(unused_imports)]
use cortex_m::{asm::bkpt, interrupt::Mutex, iprint, iprintln, peripheral::ITM, singleton};
use embedded_hal::digital::v2::OutputPin;
#[allow(unused_imports)]
use nb::block;

type LedPin = gpioc::PC13<Output<PushPull>>;
static _LED: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));

mod system_timer;
use system_timer::SystemTimer;
mod timer;
use timer::{CounterTypeExt, Timer};
mod usart_adapter;
use usart_adapter::UsartAdapter;

#[interrupt]
fn USART1() {}

#[entry]
fn main() -> ! {
    //let cp = cortex_m::Peripherals::take().unwrap();
    let dp = Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpio = dp.GPIOC.split(&mut rcc.apb2);
    let mut led: LedPin = gpio.pc13.into_push_pull_output(&mut gpio.crh);
    led.set_high().unwrap();
    SystemTimer::init(dp.TIM2, &clocks, &mut rcc.apb1);
    //let mut itm = cp.ITM;
    //iprintln!(&mut itm.stim[0], "hello wordl!");

    // USART1
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let txp = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rxp = gpioa.pa10;
    let channels = dp.DMA1.split(&mut rcc.ahb);
    let serial = Serial::usart1(
        dp.USART1,
        (txp, rxp),
        &mut afio.mapr,
        serial::Config::default().baudrate(19200.bps()),
        clocks,
        &mut rcc.apb2,
    );
    let mut adapter = UsartAdapter::new(serial, channels);
    adapter.write_data(b"hello");
    adapter.write_data(b"hello2");

    let mut t = Timer::new();
    loop {
        if t.every(1.sec()) {
            cortex_m::interrupt::free(|_| {
                led.toggle().unwrap();
            })
        }
    }
}
