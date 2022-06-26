#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_main]
#![no_std]

use panic_rtt_target as _;

use stm32f7xx_hal as hal;

use crate::hal::{adc::Adc, adc::SampleTime, delay::Delay, pac, prelude::*};
use core::fmt::Write;
use cortex_m_rt::entry;
use micromath::F32Ext;
use reed_solomon::Decoder;
use rtt_target::rtt_init;
use tinyvec::ArrayVec;

const MESSAGE: [u8; 2] = *b"He";
const DELAY: u32 = 100_000;
const ECC_LENGTH: usize = 1;

const CUT_OFF: u16 = 350;
const TEST_DELAY: u32 = DELAY / 2;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum State {
    WaitingForStart1,
    WaitingForStart0,
    Receiving,
    SendingResult,
}

#[entry]
fn main() -> ! {
    let channels = rtt_init! {
        up: {
            0: {
                size: 1024
                    name: "Terminal"
            }
        }
    };
    let mut channel = channels.up.0;
    let p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();
    let gpioc = p.GPIOC.split();

    let mut red = gpiob.pb14.into_push_pull_output();
    let mut green = gpiob.pb0.into_push_pull_output();
    let mut blue = gpiob.pb7.into_push_pull_output();

    let mut adc_in = gpioa.pa3.into_analog();

    let btn = gpioc.pc13.into_floating_input();

    // Set up the system clock. We want to run at 216MHz for this one.
    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

    // Create a delay abstraction based on SysTick
    let mut delay = Delay::new(cp.SYST, clocks);

    let adc = p.ADC1;
    let mut apb = rcc.apb2;

    let mut adc = Adc::adc1(adc, &mut apb, clocks, 12, false);

    adc.set_sample_time(SampleTime::T_3);

    let dec = Decoder::new(ECC_LENGTH);

    let mut state = State::WaitingForStart1;
    let mut received_message = ArrayVec::<[u8; 100]>::default();
    red.set_high();

    write!(&mut channel, "Started listening!\n").unwrap();

    loop {
        if btn.is_low() {
            match state {
                State::WaitingForStart1 => {
                    red.set_high();
                    let val: u16 = adc.read(&mut adc_in).unwrap();
                    if val > CUT_OFF {
                        state = State::WaitingForStart0;
                        red.set_low();
                        delay.delay_us(DELAY);
                    } else {
                        delay.delay_us(TEST_DELAY);
                    }
                }
                State::WaitingForStart0 => {
                    blue.set_high();
                    let val: u16 = adc.read(&mut adc_in).unwrap();
                    if val > CUT_OFF {
                        delay.delay_us(DELAY);
                    } else {
                        state = State::Receiving;
                        blue.set_low();
                        delay.delay_us(DELAY);
                    }
                }
                State::Receiving => {
                    green.set_high();
                    let mut received_byte = 0;
                    for idx in 0..8 {
                        let mut sum = 0;
                        for _ in 0..3 {
                            let val: u16 = adc.read(&mut adc_in).unwrap();
                            if val > CUT_OFF {
                                sum += 1;
                            }
                            delay.delay_us(DELAY);
                        }
                        if sum > 1 {
                            received_byte = received_byte | (1 << idx);
                        }
                    }
                    received_message.push(received_byte);
                    if received_message.len() >= MESSAGE.len() + ECC_LENGTH {
                        state = State::SendingResult;
                        green.set_low();
                        delay.delay_us(DELAY);
                    } else {
                        state = State::WaitingForStart1;
                        green.set_low();
                        delay.delay_us(DELAY);
                    }
                }
                State::SendingResult => {
                    let decoded = dec.correct(&mut received_message, None);
                    match decoded {
                        Ok(msg) => {
                            write!(&mut channel, "Received message: ").unwrap();
                            channel.write(msg.data());
                            write!(&mut channel, "\n",).unwrap();
                        }
                        Err(_) => {
                            write!(
                                &mut channel,
                                "{:?}\n",
                                received_message
                            )
                            .unwrap();
                        }
                    }
                    state = State::WaitingForStart1;
                    received_message.clear();
                }
            }
        } else {
            let val: u16 = adc.read(&mut adc_in).unwrap();
            write!(&mut channel, "ADC: {}\n", val).unwrap();
            delay.delay_ms(100_u32);
        }
    }
}
