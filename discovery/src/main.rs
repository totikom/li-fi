#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_main]
#![no_std]

//#![allow(unused_variables)]
//#![allow(dead_code)]

use panic_rtt_target as _;

use stm32f3xx_hal::{delay::Delay, pac, prelude::*};

use cortex_m_rt::entry;

use reed_solomon::Encoder;
use rtt_target::{rtt_init_print, rprintln};

const MESSAGE: [u8; 11] = *b"Hello, led!";
const DELAY: u32 = 10_000;
const ECC_LENGTH: usize = 8;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();

    // Set up the system clock. We want to run at 48MHz for this one.
    let mut rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.MHz()).freeze(&mut flash.acr);

    let mut gpioe = p.GPIOE.split(&mut rcc.ahb);
    let mut gpioa = p.GPIOA.split(&mut rcc.ahb);

    let mut red = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut green = gpioe
        .pe15
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut blue = gpioe
        .pe12
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    let mut led = gpioe
        .pe7
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    let btn = gpioa.pa0.into_input(&mut gpioa.moder);

    // Create a delay abstraction based on SysTick
    let mut delay = Delay::new(cp.SYST, clocks);

    let enc = Encoder::new(ECC_LENGTH);

    let enc_message = enc.encode(&MESSAGE);

    let enc_message = *enc_message;

    loop {
        if btn.is_low().unwrap() {
            red.set_high().unwrap();
            for byte in enc_message.iter() {
                blue.set_high().unwrap();

                led.set_high().unwrap();
                delay.delay_us(DELAY);
                led.set_low().unwrap();
                delay.delay_us(DELAY);

                blue.set_low().unwrap();
                green.set_high().unwrap();
                for idx in 0..8 {
                    if byte & (1 << idx) != 0 {
                        led.set_high().unwrap();
                    } else {
                        led.set_low().unwrap();
                    }

                    delay.delay_us(3*DELAY);
                }
                led.set_low();
                delay.delay_us(DELAY);
                green.set_low().unwrap();
            }
            red.set_low().unwrap();
            rprintln!("{:?}", enc_message);
            delay.delay_us((MESSAGE.len() + ECC_LENGTH) as u32 * DELAY * (2 + 8 * 3 + 1) * 2);
        } else {
            led.toggle();
            delay.delay_ms(500_u32);
        }
    }
}
