use core::fmt::Write;
use embedded_graphics::image::Image;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, StyledDrawable};
use heapless::String;

use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_nrf::{gpio::Output, peripherals::SPI3, spim::Spim};
use embassy_time::Delay;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::text::Text;
use embedded_graphics::{mono_font::MonoTextStyle, prelude::*};
use embedded_graphics::pixelcolor::Rgb565;
use mipidsi::{interface::SpiInterface, models::ST7789, options::{ColorInversion, Orientation, Rotation}, Builder};
use tinybmp::Bmp;

use crate::model::to_seconds;
use crate::{ CHANNEL, SHARED_STATE };

#[embassy_executor::task]
pub async fn display_task(
    spi_device: SpiDevice<'static, embassy_sync::blocking_mutex::raw::NoopRawMutex, Spim<'static, SPI3>, Output<'static>>,
    dc: Output<'static>,
    rst: Output<'static>,
) {
    // initialize the display
    let mut delay = Delay.clone();
    let mut buffer = [0_u8; 1024];
    let di = SpiInterface::new(spi_device, dc, &mut buffer);
    let mut display = Builder::new(ST7789, di)
        .display_size(240, 240)
        .reset_pin(rst)
        .invert_colors(ColorInversion::Inverted)
        .orientation(Orientation::new().rotate(Rotation::Deg270))
        .init(&mut delay)
        .unwrap();

    let character_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);

    // let bmp_data = include_bytes!("../img/bg.bmp");
    // let bmp: Bmp<'_, Rgb565> = Bmp::from_slice(bmp_data).unwrap();
    // let bg = Image::new(&bmp, Point::new(0, 0));

    display.clear(Rgb565::BLACK).unwrap();

    loop {
        CHANNEL.receive().await;

        {        
            let mut container = SHARED_STATE.lock().await;
            if let Some(state) = container.get_mut() {
                let duration = to_seconds(state.millis_left);

                let minutes = duration / 60;
                let seconds = duration % 60;
        
                let mut write: String<1024> = String::new();
                core::write!(&mut write, "State: {:?}\nTarget: {}:00\nSeconds Left: {:02}:{:02}", state.state, state.target_minutes, minutes, seconds).unwrap();
            
                // Create a text at position (20, 30) and draw it using the previously defined style.
                let text = Text::new(
                    &write,
                    Point::new(20, 30),
                    character_style
                );

                text.bounding_box()
                    .draw_styled(&PrimitiveStyleBuilder::new().fill_color(Rgb565::BLACK).build(), &mut display)
                    .unwrap();

                text.draw(&mut display).unwrap();       
            }
        }
    }
}