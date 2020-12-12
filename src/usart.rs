extern crate hal;
use crate::hal::pac::interrupt;
use crate::hal::prelude::*;
use crate::hal::{
    pac::{GPIOA, USART1},
    serial::{self, Serial},
};

use crate::hardware::usart_adapter::UsartAdapter;
use crate::utils::global_cell::GlobalCell;

pub static _USART: GlobalCell<UsartAdapter> = GlobalCell::<UsartAdapter>::new();

pub fn create_adapter(
    usart1: USART1,
    mut mapr: &mut hal::afio::MAPR,
    //pa9: gpioa::PA9<Input<Floating>>,
    //pa10: gpioa::PA10<Input<Floating>>,
    // mut crh: &mut gpioa::CRH,
    gpioa: GPIOA,
    channels: hal::dma::dma1::Channels,
    clocks: hal::rcc::Clocks,
    mut apb2: &mut hal::rcc::APB2,
) -> UsartAdapter {
    let mut gpioa = gpioa.split(&mut apb2);
    let txp = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rxp = gpioa.pa10;
    let serial = Serial::usart1(
        usart1,
        (txp, rxp),
        &mut mapr,
        serial::Config::default().baudrate(9600.bps()),
        clocks,
        &mut apb2,
    );
    UsartAdapter::new(serial, channels)
}

#[interrupt]
fn USART1() {
    _USART.get().isr_handler();
}
