use std::thread;
use std::time::Duration;

use display_interface_spi::SPIInterfaceNoCS;
use embedded_hal::spi::MODE_3;

use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::*;
use esp_idf_hal::units::FromValueType;

// use embedded_graphics::image::*;
use embedded_graphics::pixelcolor::Rgb666;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Ellipse, Line, PrimitiveStyle};

use mipidsi::{Builder, ColorOrder, Orientation};

fn main() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let spi = peripherals.spi2;

    let rst = PinDriver::output(peripherals.pins.gpio5)?;
    let dc = PinDriver::output(peripherals.pins.gpio32)?;
    let mut backlight = PinDriver::output(peripherals.pins.gpio4)?;
    let sclk = peripherals.pins.gpio18;
    let sda = peripherals.pins.gpio23;
    let sdi = peripherals.pins.gpio12;
    let cs = peripherals.pins.gpio27;

    let mut button = PinDriver::input(peripherals.pins.gpio22)?;
    button.set_pull(Pull::Down)?;
    let mut delay = Ets;

    let config = config::Config::new()
        .baudrate(26.MHz().into())
        .data_mode(MODE_3);

    let device = SpiDeviceDriver::new_single(
        spi,
        sclk,
        sda,
        Some(sdi),
        Some(cs),
        &SpiDriverConfig::new(),
        &config,
    )?;

    // display interface abstraction from SPI and DC
    let di = SPIInterfaceNoCS::new(device, dc);

    // create driver
    let mut display = Builder::ili9341_rgb666(di)
        .with_display_size(240, 320)
        .with_orientation(Orientation::Portrait(true))
        .with_color_order(ColorOrder::Bgr)
        .init(&mut delay, Some(rst))
        .unwrap();

    // turn on the backlight
    backlight.set_high()?;
    // let raw_image_data = imagerawle::new(include_bytes!("../examples/assets/ferris.raw"), 86);
    // let ferris = Image::new(&raw_image_data, Point::new(0, 0));

    // ferris.draw(&mut display).unwrap();

    println!("Image printed!");
    display.clear(Rgb666::BLACK).unwrap();

    let mut y = 0;
    loop {
        // Point (horizontal 0..N, vertical 0..M)
        Ellipse::new(Point::new(8, 16 + y - 1), Size::new(48, 32))
            .into_styled(PrimitiveStyle::with_stroke(Rgb666::BLACK, 2))
            .draw(&mut display)
            .unwrap();
        Ellipse::new(Point::new(8, 16 + y), Size::new(48, 32))
            .into_styled(PrimitiveStyle::with_stroke(Rgb666::YELLOW, 2))
            .draw(&mut display)
            .unwrap();
        thread::sleep(Duration::from_millis(10));
        if y < 120 {
            y += 1;
        }
        println!("Button pushed {}", button.is_high());
    }
}
