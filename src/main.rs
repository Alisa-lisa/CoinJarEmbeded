use std::sync::Arc;
use std::time::{Duration, Instant};

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::mono_font::ascii::FONT_8X13;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::text::Text;
use embedded_hal::spi::MODE_3;

use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::*;
use esp_idf_hal::units::FromValueType;

// use embedded_graphics::image::*;
use embedded_graphics::pixelcolor::Rgb666;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Ellipse, PrimitiveStyle};

use mipidsi::{Builder, ColorOrder, Orientation};

use tokio::join;
use tokio::sync::Mutex;

static WIDTH: u16 = 240;
static HEIGHT: u16 = 320;

enum CoinType {
    GoodCoin,
    BadCoin,
}

struct Coin {
    position: Point,
    coin_type: CoinType,
}

impl Coin {
    fn new(coin_type: CoinType) -> Self {
        let position = Point::new(WIDTH as i32 / 2, 20);
        Self {
            position,
            coin_type,
        }
    }

    fn draw<D>(&self, display: &mut D)
    where
        D: DrawTarget<Color = Rgb666>,
        D::Error: std::fmt::Debug,
    {
        let color = match self.coin_type {
            CoinType::GoodCoin => Rgb666::YELLOW,
            CoinType::BadCoin => Rgb666::RED,
        };

        Ellipse::with_center(self.position, Size::new(48, 32))
            .into_styled(PrimitiveStyle::with_stroke(color, 2))
            .draw(display)
            .unwrap();
    }
}

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

    let mut button_center = PinDriver::input(peripherals.pins.gpio37)?;
    let mut button_left = PinDriver::input(peripherals.pins.gpio38)?;
    let mut button_right = PinDriver::input(peripherals.pins.gpio39)?;
    let mut button_side = PinDriver::input(peripherals.pins.gpio0)?;
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
        .with_display_size(WIDTH, HEIGHT)
        .with_orientation(Orientation::Portrait(true))
        .with_color_order(ColorOrder::Bgr)
        .init(&mut delay, Some(rst))
        .unwrap();

    // turn on the backlight
    backlight.set_high()?;

    println!("Image printed!");

    let coins = Arc::new(Mutex::new(vec![]));
    let mut last_fps = 0;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        let coins1 = coins.clone();
        let left = tokio::spawn(async move {
            loop {
                button_left.wait_for_low().await.unwrap();
                coins1.lock().await.push(Coin::new(CoinType::BadCoin));
            }
        });

        let coins2 = coins.clone();
        let right = tokio::spawn(async move {
            loop {
                button_right.wait_for_low().await.unwrap();
                coins2.lock().await.push(Coin::new(CoinType::GoodCoin));
            }
        });

        let coins3 = coins.clone();
        let draw_loop = tokio::spawn(async move {
            loop {
                let now = Instant::now();

                display.clear(Rgb666::BLACK).unwrap();

                let style = MonoTextStyle::new(&FONT_8X13, Rgb666::WHITE);
                Text::new(
                    &format!("FPS: {}", last_fps),
                    Point::new(WIDTH as i32 - 80, 10),
                    style,
                )
                .draw(&mut display)
                .unwrap();

                let coins_ = coins3.lock().await;
                for coin in coins_.iter() {
                    coin.draw(&mut display);
                }

                last_fps = 1000 / now.elapsed().as_millis();

                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });

        let _ = join!(left, right, draw_loop);
        Ok(())
    })
}
