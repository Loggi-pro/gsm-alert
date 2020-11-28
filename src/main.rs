#![no_std]
#![no_main]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust#53964
extern crate panic_halt; // panic hnadler
use cortex_m_rt::entry;
extern crate hal;
use crate::hal::{
    gpio::{gpioc,Output,PushPull},
    pac::{Peripherals},
    prelude::*,
};
use core::cell::RefCell;

#[allow(unused_imports)]
use cortex_m::{asm::bkpt, iprint, iprintln, peripheral::ITM,interrupt::Mutex};
use embedded_hal::digital::v2::OutputPin;

#[allow(unused_imports)]
use nb::block;


type LedPin = gpioc::PC13<Output<PushPull>>;
static _LED:Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));


mod system_timer;
use system_timer::{SystemTimer};
mod timer;
use timer::{Timer,CounterTypeExt};





#[entry]
fn main() -> ! {
    //let cp = cortex_m::Peripherals::take().unwrap();
    let dp = Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpio = dp.GPIOC.split(&mut rcc.apb2);
    let mut led:LedPin = gpio.pc13.into_push_pull_output(&mut gpio.crh);
    led.set_high().unwrap();
    SystemTimer::init(dp.TIM2,&clocks,&mut rcc.apb1);
    //let mut itm = cp.ITM;
    
    //iprintln!(&mut itm.stim[0], "hello wordl!");


    // Ждём пока таймер запустит обновление
    // и изменит состояние светодиода.
    let mut t = Timer::new();
    loop {
        if t.every(1.sec()) {
            cortex_m::interrupt::free(|_|{
                led.toggle().unwrap();
            })
        }
    }


}
