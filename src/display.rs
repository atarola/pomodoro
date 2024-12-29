use core::fmt::Write;
use heapless::String;

use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_nrf::{gpio::Output, peripherals::TWISPI0, spim::Spim};
use embassy_time::Delay;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::text::{Text, TextStyleBuilder};
use embedded_graphics::{mono_font::MonoTextStyle, prelude::*};
use embedded_graphics::pixelcolor::Rgb565;
use mipidsi::{interface::SpiInterface, models::ST7789, options::{ColorInversion, Orientation, Rotation}, Builder};

use crate::{SharedState, CHANNEL, SHARED_STATE};

#[embassy_executor::task]
pub async fn display_task(
    spi_device: SpiDevice<'static, embassy_sync::blocking_mutex::raw::NoopRawMutex, Spim<'static, TWISPI0>, Output<'static>>,
    dc: Output<'static>,
    rst: Output<'static>,
) {
    // initialize the display
    let mut delay = Delay.clone();
    let mut buffer = [0_u8; 512];
    let di = SpiInterface::new(spi_device, dc, &mut buffer);
    let mut display = Builder::new(ST7789, di)
        .display_size(240, 240)
        .reset_pin(rst)
        .invert_colors(ColorInversion::Inverted)
        .orientation(Orientation::new().rotate(Rotation::Deg270))
        .init(&mut delay)
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    // draw the things
    // Create a new character style.
    let character_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);

    // Create a new text style.
    let text_style = TextStyleBuilder::new().build();
    let data: SharedState; 

    {        
        let mut container = SHARED_STATE.lock().await;
        data = match container.get_mut() {
            Some(x) => x.clone(),
            None => SharedState::new()
        };
    }

    let mut write: String<1024> = String::new();
    core::write!(&mut write, "Hello!\nTarget Minutes: {}\nTicks Left: {}", data.target_minutes, data.ticks_left).unwrap();

    // Create a text at position (20, 30) and draw it using the previously defined style.
    Text::with_text_style(
        &write,
        Point::new(20, 30),
        character_style,
        text_style,
    )
    .draw(&mut display)
    .unwrap();

    loop {
        CHANNEL.receive().await;
    }
}