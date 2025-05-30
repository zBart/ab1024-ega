#![no_std]
#![no_main]

//! This example demonstrates using the driver without embedded-graphics
//!
//! ![Inkplate displaying a solid rectangle for each color in AB1024-EGA palette.  The rectangles are black, white, green, blue, red, yellow and orange.][no_graphics]
//!
#![doc = ::embed_doc_image::embed_image!("no_graphics", "examples/no_graphics_photo.jpg")]
//!

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

    let colors = [
        ab1024_ega::color::Color::BLACK,
        ab1024_ega::color::Color::WHITE,
        ab1024_ega::color::Color::GREEN,
        ab1024_ega::color::Color::BLUE,
        ab1024_ega::color::Color::RED,
        ab1024_ega::color::Color::YELLOW,
        ab1024_ega::color::Color::ORANGE,
    ];
    let mut display = ab1024_ega::Display::new(spi, rst, dc, busy, delay);
    for (index, color) in colors.into_iter().enumerate() {
        for x in (index * ab1024_ega::WIDTH / colors.len())..ab1024_ega::WIDTH {
            for y in 0..ab1024_ega::HEIGHT {
                display.set_pixel(x, y, color).unwrap();
            }
        }
    }

    display.init().unwrap();
    display.display().unwrap();

    Rtc::new(peripherals.LPWR).sleep_deep(&[])
}
