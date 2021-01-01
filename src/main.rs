#![no_std]
#![no_main]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust#53964
extern crate panic_halt; // panic hnadler
use cortex_m_rt::entry;
extern crate hal;
use crate::hal::{
    gpio::{gpiob, Output, PushPull, Pxx},
    pac::Peripherals,
    prelude::*,
};

#[allow(unused_imports)]
use cortex_m::{asm::bkpt, iprint, iprintln, peripheral::ITM, singleton};
use embedded_hal::digital::v2::OutputPin;
#[allow(unused_imports)]
use nb::block;

type Sim900PowerPin = gpiob::PB5<Output<PushPull>>;
mod utils;

mod sim900;
use sim900::Sim900;

mod hardware;
mod timer;
use timer::{CounterTypeExt, Timer};
mod indication;
mod usart;
use indication::{Indication, IndicationState};

#[entry]
fn main() -> ! {
    //let cp = cortex_m::Peripherals::take().unwrap();
    let dp = Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let power_pin: Sim900PowerPin = gpiob.pb5.into_push_pull_output(&mut gpiob.crl);
    let led_red = gpioa.pa11.into_push_pull_output(&mut gpioa.crh).downgrade();
    let led_green = gpioa.pa12.into_push_pull_output(&mut gpioa.crh).downgrade();
    //_LED.set(led);
    Timer::init_system(dp.TIM2, &clocks, &mut rcc.apb1);
    //let mut itm = cp.ITM;
    //iprintln!(&mut itm.stim[0], "hello wordl!");

    // USART1
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let adapter = usart::create_adapter(
        dp.USART1,
        &mut afio.mapr,
        gpioa.pa9,
        gpioa.pa10,
        &mut gpioa.crh,
        dp.DMA1.split(&mut rcc.ahb),
        clocks,
        &mut rcc.apb2,
    );
    usart::_USART.set(adapter);
    let mut sim900 = Sim900::new(power_pin.downgrade());
    sim900.init();
    let mut sim900 = sim900.power_on();
    let _ = sim900.setup();
    let r = sim900.get_state();
    //sim900.send_pdu_sms(
    //    "0001000B919741123274F200082E0422044004350432043E043304300021000A0414043204350440044C0020043E0442043A0440044B044204300021",
    //);
    let r2 = sim900.get_state();
    let mut indication: Indication = Indication::new(led_red, led_green);
    indication.set_state(IndicationState::Demo);
    loop {
        indication.poll();
    }
}
