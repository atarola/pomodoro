use apds9960::Apds9960;
use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_nrf::{gpio::Input, peripherals::TWISPI1, twim::Twim};
use embassy_time::{Duration, Ticker, Timer};

use crate::{DISPLAY_SIGNAL, SHARED_STATE};

#[embassy_executor::task]
pub async fn button_a_task(mut pin: Input<'static>) {
    loop {
        pin.wait_for_low().await;

        {        
            let mut container = SHARED_STATE.lock().await;
            if let Some(state) = container.get_mut() {
                state.target_down();
            }
        }

        DISPLAY_SIGNAL.send(true).await;
        Timer::after(Duration::from_millis(150)).await;
    }
}

#[embassy_executor::task]
pub async fn button_b_task(mut pin: Input<'static>) {
    loop {
        pin.wait_for_low().await;

        {        
            let mut container = SHARED_STATE.lock().await;
            if let Some(state) = container.get_mut() {
                state.target_up();
            }
        }

        DISPLAY_SIGNAL.send(true).await;
        Timer::after(Duration::from_millis(150)).await;
    }
}

#[embassy_executor::task]
pub async fn proximity_sensor_task(
    i2c_device: I2cDevice<'static, embassy_sync::blocking_mutex::raw::NoopRawMutex, Twim<'static, TWISPI1>>,
) {
    let mut sensor = Apds9960::new(i2c_device);
    sensor.enable().unwrap();
    sensor.enable_proximity().unwrap();
    sensor.enable_proximity_interrupts().unwrap();

    let mut ticker = Ticker::every(Duration::from_millis(100));

    loop {
        ticker.next().await;

        if let core::result::Result::Ok(data) = sensor.read_proximity() {
            if data > 200 {
                {
                    let mut container = SHARED_STATE.lock().await;
                    if let Some(state) = container.get_mut() {
                        state.toggle();
                        DISPLAY_SIGNAL.send(true).await;
                    }
                }

                Timer::after(Duration::from_millis(150)).await;
            }
        }
    }
}