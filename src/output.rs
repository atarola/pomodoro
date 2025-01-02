use core::fmt::Write;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_nrf::{gpio::Output, peripherals::SPI3, spim::Spim};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::mono_font::ascii::{FONT_10X20, FONT_7X13};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::*;
use embedded_graphics::text::Text;
use embedded_graphics::{mono_font::MonoTextStyle, prelude::*};
use embedded_layout::layout::linear::FixedMargin;
use embedded_layout::{layout::linear::LinearLayout, prelude::*};
use heapless::String;
use mipidsi::{
    interface::SpiInterface,
    models::ST7789,
    options::{ColorInversion, Orientation, Rotation},
    Builder,
};

use crate::model::{to_seconds, CurrentState};
use crate::{DISPLAY_SIGNAL, SHARED_STATE, STATE_SIGNAL};

#[embassy_executor::task]
pub async fn display_task(
    spi_device: SpiDevice<
        'static,
        embassy_sync::blocking_mutex::raw::NoopRawMutex,
        Spim<'static, SPI3>,
        Output<'static>,
    >,
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

    let time_left_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    let target_style = MonoTextStyle::new(&FONT_7X13, Rgb565::CSS_DARK_GRAY);
    let divider_style = PrimitiveStyle::with_stroke(Rgb565::CSS_DARK_GRAY, 1);
    let stopped_style = PrimitiveStyle::with_stroke(Rgb565::CSS_DARK_RED, 3);
    let started_style = PrimitiveStyle::with_stroke(Rgb565::CSS_DARK_GREEN, 3);
    let clear_style = PrimitiveStyle::with_fill(Rgb565::BLACK);

    display.clear(Rgb565::BLACK).unwrap();

    loop {
        DISPLAY_SIGNAL.receive().await;

        {
            let mut container = SHARED_STATE.lock().await;
            if let Some(state) = container.get_mut() {
                let duration = to_seconds(state.millis_left);
                let minutes = duration / 60;
                let seconds = duration % 60;

                let mut time_left_text: String<5> = String::new();
                core::write!(&mut time_left_text, "{:02}:{:02}", minutes, seconds).unwrap();
                let time_left = Text::new(&time_left_text, Point::zero(), time_left_style);

                let divider =
                    Line::new(Point::zero(), Point::new(128, 0)).into_styled(divider_style);

                let mut target_text: String<5> = String::new();
                core::write!(&mut target_text, "{:02}:00", state.target_minutes).unwrap();
                let target = Text::new(&target_text, Point::zero(), target_style);

                let layout =
                    LinearLayout::vertical(Chain::new(time_left).append(divider).append(target))
                        .with_alignment(horizontal::Center)
                        .with_spacing(FixedMargin(10))
                        .arrange()
                        .align_to(
                            &display.bounding_box(),
                            horizontal::Center,
                            vertical::Center,
                        );

                let bounding_rect = layout.bounds().offset(6);
                bounding_rect
                    .draw_styled(&clear_style, &mut display)
                    .unwrap();

                if state.state == CurrentState::STARTED {
                    bounding_rect
                        .draw_styled(&started_style, &mut display)
                        .unwrap();
                } else {
                    bounding_rect
                        .draw_styled(&stopped_style, &mut display)
                        .unwrap();
                }

                layout.draw(&mut display).unwrap();
            }
        }
    }
}

#[embassy_executor::task]
pub async fn notification_task(mut spk: Output<'static>) {
    loop {
        STATE_SIGNAL.receive().await;

        for _ in 1..5 {
            spk.set_high();
            Timer::after(Duration::from_millis(500)).await;
            spk.set_low();
            Timer::after(Duration::from_millis(2)).await;
        }
    }
}
