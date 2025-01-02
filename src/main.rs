#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

mod output;
mod model;
mod input;

use core::cell::RefCell;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputDrive, Pull},
    peripherals::{SPI3, TWISPI1},
    spim::{self, Frequency, Spim}, twim::{self, Twim}
};
use embassy_sync::{blocking_mutex::{raw::ThreadModeRawMutex, NoopMutex}, channel::Channel, mutex::Mutex};
use embassy_time::{Duration, Ticker};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SPIM3 => spim::InterruptHandler<SPI3>;
    SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => twim::InterruptHandler<TWISPI1>;
});

static DISPLAY_SIGNAL: Channel<ThreadModeRawMutex, bool, 1> = Channel::new();
static STATE_SIGNAL: Channel<ThreadModeRawMutex, bool, 1> = Channel::new();
static SPI_BUS: StaticCell<NoopMutex<RefCell<Spim<SPI3>>>> = StaticCell::new();
static I2C_BUS: StaticCell<NoopMutex<RefCell<Twim<TWISPI1>>>> = StaticCell::new();
static SHARED_STATE: Mutex<ThreadModeRawMutex, RefCell<Option<model::SharedState>>> = Mutex::new(RefCell::new(Option::None));

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    // pins
    let button_a = Input::new(p.P1_02, Pull::Up);
    let button_b = Input::new(p.P1_10, Pull::Up);
    let cs = Output::new(p.P0_12, Level::Low, OutputDrive::Standard);
    let dc = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);
    let rst = Output::new(p.P1_03, Level::Low, OutputDrive::Standard);
    let spk = Output::new(p.P1_00, Level::Low, OutputDrive::Standard);

    // data
    {
        let data = model::SharedState::new();
        let container = SHARED_STATE.lock().await;
        container.borrow_mut().replace(data);
    }

    // SPI for display
    let mut spi_config = spim::Config::default();
    spi_config.frequency = Frequency::M32;
    spi_config.mode = spim::MODE_3;
    spi_config.orc = 122;

    let spim = spim::Spim::new_txonly(p.SPI3, Irqs, p.P0_14, p.P0_15, spi_config);
    let spi_bus = NoopMutex::new(RefCell::new(spim));
    let spi_bus = SPI_BUS.init(spi_bus);
    let spi_dev = SpiDevice::new(spi_bus, cs);
    
    // I2C for sensors
    let i2c_config = twim::Config::default();
    let twim = twim::Twim::new(p.TWISPI1, Irqs, p.P0_24, p.P0_25, i2c_config);
    let i2c_bus = NoopMutex::new(RefCell::new(twim));
    let i2c_bus = I2C_BUS.init(i2c_bus);
    let i2c_dev = I2cDevice::new(i2c_bus);

    // tasks
    spawner.must_spawn(input::button_a_task(button_a));
    spawner.must_spawn(input::button_b_task(button_b));
    spawner.must_spawn(input::proximity_sensor_task(i2c_dev));
    spawner.must_spawn(output::display_task(spi_dev, dc, rst));
    spawner.must_spawn(output::notification_task(spk));

    // heartbeat
    let mut ticker = Ticker::every(Duration::from_millis(300));

    loop {
        ticker.next().await;
        
        {        
            let mut container = SHARED_STATE.lock().await;
            if let Some(state) = container.get_mut() {
                let (second_changed, state_changed) = state.tick();

                if state_changed {
                    STATE_SIGNAL.send(true).await;
                }

                if second_changed {
                    DISPLAY_SIGNAL.send(true).await;
                }
            } 
        }

    }
}
