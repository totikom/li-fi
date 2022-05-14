#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

use stm32f7xx_hal as hal;

use crate::hal::{adc::Adc, delay::Delay, pac, prelude::*};
use cortex_m_rt::entry;
use micromath::F32Ext;
use rtt_target::{rtt_init};
use tinyvec::ArrayVec;
use core::fmt::Write;

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

    let gpiob = p.GPIOB.split();

    let mut red = gpiob.pb14.into_push_pull_output();
    let mut green = gpiob.pb0.into_push_pull_output();
    //let mut blue = gpiob.pb7.into_push_pull_output();

    let mut a6_in = gpiob.pb1.into_analog();

    // Set up the system clock. We want to run at 48MHz for this one.
    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

    // Create a delay abstraction based on SysTick
    let mut delay = Delay::new(cp.SYST, clocks);

    let adc = p.ADC1;
    let mut apb = rcc.apb2;

    let mut adc = Adc::adc1(adc, &mut apb, clocks, 12, false);

    const sampling_frame: usize = 100;
    const message: [u8; 11] = *b"Hello, led!";
    green.set_low();

    let intervals: [u32; 15] = [1, 10, 100, 200, 300, 400, 500, 1000, 2000, 4000, 8000, 16000, 32000, 64000, 128000];

    let mut high = ArrayVec::<[u16; sampling_frame]>::default();
    let mut low = ArrayVec::<[u16; sampling_frame]>::default();

    let mut received_message = ArrayVec::<[u8; 11]>::default();
    for interval in intervals {
        high.clear();
        low.clear();
        received_message.clear();

        for i in 0..sampling_frame {
            green.set_low();
            delay.delay_us(interval);

            let val: u16 = adc.read(&mut a6_in).unwrap();
            low.push(val);

            green.set_high();
            delay.delay_us(interval);

            let val: u16 = adc.read(&mut a6_in).unwrap();
            high.push(val);
            red.toggle();
            //write!(&mut channel,"{}", i);
        }

        let high_mean = high.iter().sum::<u16>() as f32 / high.len() as f32;
        let high_std = (high
            .iter()
            .fold(0.0, |acc, &x| acc + (x as f32 - high_mean).powi(2))
            / high.len() as f32)
            .sqrt();

        let low_mean = low.iter().sum::<u16>() as f32 / low.len() as f32;
        let low_std = (low
            .iter()
            .fold(0.0, |acc, &x| acc + (x as f32 - low_mean).powi(2))
            / low.len() as f32)
            .sqrt();

        let border = (high_mean + low_mean) as u16 / 2;

        for byte in message {
            let mut received_byte = 0;
            for idx in 0..7 {
                if byte & (1 << idx) != 0 {
                    green.set_high();
                } else {
                    green.set_low();
                }

                delay.delay_us(interval);

                let val: u16 = adc.read(&mut a6_in).unwrap();
                if val > border {
                    received_byte = received_byte | (1 << idx);
                }
            }
            received_message.push(received_byte);
        }
        write!(&mut channel,
            "interval: {}us, high mean: {} +/- {}, low mean: {} +/- {}, message: ",
            interval,
            high_mean,
            high_std,
            low_mean,
            low_std,
        );
        channel.write(&received_message);
        write!(&mut channel, "\n");
    }

    loop {
        red.toggle();
        delay.delay_ms(500u32);
    }
}
