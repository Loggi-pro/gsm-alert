#![no_std]
#![no_main]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust#53964
extern crate panic_halt; // panic hnadler
use cortex_m_rt::entry;
extern crate hal;
use crate::hal::{
    gpio::{gpiob, gpioc, Output, PushPull},
    pac::Peripherals,
    prelude::*,
};

#[allow(unused_imports)]
use cortex_m::{asm::bkpt, iprint, iprintln, peripheral::ITM, singleton};
use embedded_hal::digital::v2::OutputPin;
#[allow(unused_imports)]
use nb::block;

type LedPin = gpioc::PC13<Output<PushPull>>;
type Sim900PowerPin = gpiob::PB5<Output<PushPull>>;
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
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let power_pin: Sim900PowerPin = gpiob.pb5.into_push_pull_output(&mut gpiob.crl);
    let mut led: LedPin = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
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
    let mut sim900 = Sim900::new(power_pin.downgrade());
    sim900.init();
    let mut sim900 = sim900.power_on();
    let mut a = 0;
    match sim900.setup() {
        Ok(_) => a = 1,
        Err(sim900::RequestError::ENoDevice) => a = 2,
        Err(sim900::RequestError::ENoAnswer) => a = 3,
        Err(sim900::RequestError::EAnswerError) => a = 4,
        Err(sim900::RequestError::ETimeout) => a = 5,
        Err(sim900::RequestError::EBadRequest) => a = 6,
        Err(sim900::RequestError::EAnswerUnknown) => a = 7,
    };
    a = a + 1 - 1;
    let r = sim900.get_state();
    let mut t = Timer::new();
    loop {
        if t.every(1.sec()) {
            cortex_m::interrupt::free(|_| {
                led.toggle().unwrap();
            })
        }
    }
}
