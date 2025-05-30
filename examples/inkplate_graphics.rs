#![no_std]
#![no_main]

//! This example demonstrates drawing a few embedded-graphics primitives
//!
//! ![Inkplate displaying 3 circles in a row.  The last circle in the row is blue and overlaps a
//! preceding yellow circle.  The yellow circle overlaps a preceding red circle.][graphics]
//!
#![doc = ::embed_doc_image::embed_image!("graphics", "examples/graphics_photo.jpg")]
//!

use embedded_graphics::{
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::spi::{
    master::{Config, Spi},
    Mode,
};
use esp_hal::time::Rate;

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let rst = Output::new(peripherals.GPIO19, Level::Low, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO33, Level::Low, OutputConfig::default());
    let busy = Input::new(peripherals.GPIO32, InputConfig::default());
    let cs = Output::new(peripherals.GPIO27, Level::Low, OutputConfig::default());

    let spi = ExclusiveDevice::new_no_delay(
        Spi::new(
            peripherals.SPI2,
            Config::default()
                .with_frequency(Rate::from_khz(200))
                .with_mode(Mode::_0),
        )
        .unwrap()
        .with_sck(peripherals.GPIO18)
        .with_mosi(peripherals.GPIO23),
        cs,
    )
    .unwrap();

    let mut display = ab1024_ega::Display::new(spi, rst, dc, busy, delay);

    Circle::with_center(Point::new(150, 224), 200)
        .into_styled(PrimitiveStyle::with_fill(ab1024_ega::color::Color::RED))
        .draw(&mut display)
        .unwrap();
    Circle::with_center(Point::new(300, 224), 200)
        .into_styled(PrimitiveStyle::with_fill(ab1024_ega::color::Color::YELLOW))
        .draw(&mut display)
        .unwrap();
    Circle::with_center(Point::new(450, 224), 200)
        .into_styled(PrimitiveStyle::with_fill(ab1024_ega::color::Color::BLUE))
        .draw(&mut display)
        .unwrap();

    display.init().unwrap();
    display.display().unwrap();

    Rtc::new(peripherals.LPWR).sleep_deep(&[])
}
