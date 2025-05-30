#![no_std]
#![no_main]

//! This example demonstrates using the driver with a dither to approximate a display that can
//! render Rgb888 at each pixel.
//!
//! ![Inkplate displaying a dithered version of Vincent van Gogh's The Starry Night][photo]
//!
#![doc = ::embed_doc_image::embed_image!("photo", "examples/image_photo.jpg")]
//!

use ab1024_ega::color::Color;
use dither::embedded_graphics::DitherTarget;
use dither::vector::Vector;
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
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
use fixed::types::extra::U16;
use tinybmp::Bmp;
type Num = fixed::FixedI32<U16>;

const RGB_DISPLAY_PAIRS: [(Color, Rgb888); 7] = [
    (Color::BLACK, Rgb888::new(0x00, 0x00, 0x00)),
    (Color::WHITE, Rgb888::new(0xFF, 0xFF, 0xFF)),
    (Color::GREEN, Rgb888::new(0x10, 0xcb, 0x10)),
    (Color::BLUE, Rgb888::new(0x20, 0x20, 0xff)),
    (Color::RED, Rgb888::new(0xff, 0x30, 0x20)),
    (Color::YELLOW, Rgb888::new(0xff, 0xff, 0x50)),
    (Color::ORANGE, Rgb888::new(0xf0, 0x70, 0x20)),
];

fn conversion(source: Vector<Num>) -> (Color, Vector<Num>) {
    let pair = RGB_DISPLAY_PAIRS
        .into_iter()
        .min_by_key(|(_, rgb): &(Color, Rgb888)| {
            (source.0).abs_diff(Num::from_num(rgb.r()))
                + (source.1).abs_diff(Num::from_num(rgb.g()))
                + (source.2).abs_diff(Num::from_num(rgb.b()))
        })
        .unwrap();

    (pair.0, source - Vector::from_rgb888(pair.1))
}

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

    let bmp: Bmp<Rgb888> = Bmp::from_slice(include_bytes!("starry-night.bmp")).unwrap();
    let mut display = ab1024_ega::Display::new(spi, rst, dc, busy, delay);
    let mut ed: DitherTarget<'_, _, _, { ab1024_ega::WIDTH }> =
        DitherTarget::new(&mut display, &conversion);

    bmp.draw(&mut ed).unwrap();

    display.init().unwrap();
    display.display().unwrap();

    Rtc::new(peripherals.LPWR).sleep_deep(&[])
}
