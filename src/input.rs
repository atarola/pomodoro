use embassy_nrf::gpio::Input;
use embassy_time::{Duration, Timer};

use crate::CHANNEL;

#[embassy_executor::task]
pub async fn button_a_task(mut pin: Input<'static>) {
    loop {
        pin.wait_for_low().await;

        CHANNEL.send(true).await;
        Timer::after(Duration::from_millis(150)).await;
    }
}

#[embassy_executor::task]
pub async fn button_b_task(mut pin: Input<'static>) {
    loop {
        pin.wait_for_low().await;

        CHANNEL.send(true).await;
        Timer::after(Duration::from_millis(150)).await;
    }
}