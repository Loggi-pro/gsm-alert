#![no_std]
#![no_main]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust#53964
extern crate panic_halt; // panic hnadler
use cortex_m_rt::entry;
extern crate hal;
use crate::hal::{
    gpio::{gpioc, Output, PushPull},
    pac::Peripherals,
    prelude::*,
};

#[allow(unused_imports)]
use cortex_m::{asm::bkpt, iprint, iprintln, peripheral::ITM, singleton};
use embedded_hal::digital::v2::OutputPin;
#[allow(unused_imports)]
use nb::block;

type LedPin = gpioc::PC13<Output<PushPull>>;

mod utils;

mod sim900;
use sim900::Sim900;

mod hardware;
mod timer;
use timer::{CounterTypeExt, Timer};
mod usart;

#[entry]
fn main() -> ! {
    //let cp = cortex_m::Peripherals::take().unwrap();
    let dp = Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpio = dp.GPIOC.split(&mut rcc.apb2);
    let mut led: LedPin = gpio.pc13.into_push_pull_output(&mut gpio.crh);
    //_LED.set(led);
    led.set_high().unwrap();
    Timer::init_system(dp.TIM2, &clocks, &mut rcc.apb1);
    //let mut itm = cp.ITM;
    //iprintln!(&mut itm.stim[0], "hello wordl!");

    // USART1
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let adapter = usart::create_adapter(
        dp.USART1,
        &mut afio.mapr,
        dp.GPIOA,
        dp.DMA1.split(&mut rcc.ahb),
        clocks,
        &mut rcc.apb2,
    );
    usart::_USART.set(adapter);
    usart::_USART.get().write_data(b"hello");
    usart::_USART.get().write_data(b"hello2");
    let (_a, _b) = usart::_USART.get().read_data();
    let (_a, _b) = usart::_USART.get().read_data();
    let mut t = Timer::new();
    loop {
        if t.every(1.sec()) {
            cortex_m::interrupt::free(|_| {
                led.toggle().unwrap();
            })
        }
    }
}
