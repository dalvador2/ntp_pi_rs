//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::block::ImageDef;
use embassy_rp::gpio;
use embassy_rp::gpio::Input;
use embassy_rp::i2c::{Async, Instance, Mode};
use embassy_rp::peripherals::I2C0;
use embassy_rp::{bind_interrupts, i2c};
use embassy_time::Timer;
use gpio::{Level, Output, Pull};
use pwm_pca9685::{Address, Channel, Pca9685};
use {defmt_rtt as _, panic_probe as _};

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"high voltage enable"),
    embassy_rp::binary_info::rp_program_description!(
        c"This example tests enables hv on nixie clock"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
}
);

pub struct NixieState {
    digits: [char; 6],
    commas: [bool; 12],
}

impl NixieState {
    pub fn new(digits: [char; 6], commas: [bool; 12]) -> Self {
        Self { digits, commas }
    }
    pub fn blank() -> Self {
        Self {
            digits: [' '; 6],
            commas: [false; 12],
        }
    }
}
impl Default for NixieState {
    fn default() -> Self {
        NixieState {
            digits: ['0'; 6],
            commas: [false; 12],
        }
    }
}

pub struct Display<'a, T>
where
    T: Instance,
{
    current_state: NixieState,
    previous_state: NixieState,
    i2c_dev: i2c::I2c<'a, T, Async>,
}

impl<'a, T> Display<'a, T>
where
    T: Instance,
{
    fn new(i2c_dev: i2c::I2c<'a, T, Async>) -> Self {
        Display {
            current_state: NixieState::default(),
            previous_state: NixieState::blank(),
            i2c_dev,
        }
    }
    async fn wipe(mut self) -> Self {
        for i in 1u8..=6 {
            let mut pwm = Pca9685::new(self.i2c_dev, Address::from(i)).unwrap();
            pwm.enable().await.unwrap();
            pwm.set_channel_on_off(Channel::All, 0, 0).await.unwrap();
            self.i2c_dev = pwm.destroy()
        }
        self
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut gp3 = Output::new(p.PIN_3, Level::Low);
    let mut button = Input::new(p.PIN_7, Pull::Up);
    let mut ext_clk = Output::new(p.PIN_2, Level::Low);
    let mut dev = i2c::I2c::new_async(p.I2C0, p.PIN_21, p.PIN_20, Irqs, i2c::Config::default());
    for addr_int in 65u8..=69 {
        let address = Address::from(addr_int);
        let mut pwm = Pca9685::new(dev, address).unwrap();
        info!("led on!");
        // button.wait_for_low().await;
        gp3.set_high();
        ext_clk.set_low();
        pwm.set_prescale(100).await.unwrap();
        pwm.enable().await.unwrap();
        pwm.set_channel_on_off(Channel::All, 0, 0).await.unwrap();
        pwm.set_channel_on_off(Channel::try_from(7u8).unwrap(), 0, 4095)
            .await
            .unwrap();
        if addr_int == 65u8 {
            pwm.set_channel_on_off(Channel::try_from(15u8).unwrap(), 0, 4095)
                .await
                .unwrap();
        }
        Timer::after_millis(100).await;
        dev = pwm.destroy();
    }
    loop {
        Timer::after_millis(500).await;
    }
}
