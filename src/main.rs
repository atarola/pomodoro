#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

mod display;
mod input;

use core::cell::RefCell;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputDrive, Pull},
    peripherals::TWISPI0,
    spim::{self, Spim}
};
use embassy_sync::{blocking_mutex::{raw::ThreadModeRawMutex, NoopMutex}, channel::Channel, mutex::Mutex};
use embassy_time::{Duration, Ticker};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => spim::InterruptHandler<TWISPI0>;
});

#[derive(Clone)]
enum CurrentState {
    STARTED,
    STOPPED
} 

#[derive(Clone)]
struct SharedState {
    state: CurrentState,
    target_minutes: u8,
    ticks_left: u32
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            state: CurrentState::STOPPED,
            target_minutes: 20,
            ticks_left: 0
        }
    }
}

static CHANNEL: Channel<ThreadModeRawMutex, bool, 1> = Channel::new();
static SPI_BUS: StaticCell<NoopMutex<RefCell<Spim<TWISPI0>>>> = StaticCell::new();
static SHARED_STATE: Mutex<ThreadModeRawMutex, RefCell<Option<SharedState>>> = Mutex::new(RefCell::new(Option::None));

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    // data
    {
        let data = SharedState::new();
        let container = SHARED_STATE.lock().await;
        container.borrow_mut().replace(data);
    }

    // pins
    let button_a = Input::new(p.P1_02, Pull::Up);
    let button_b = Input::new(p.P1_10, Pull::Up);
    let cs = Output::new(p.P0_12, Level::Low, OutputDrive::Standard);
    let dc = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);
    let rst = Output::new(p.P1_03, Level::Low, OutputDrive::Standard);

    // SPI for display
    let mut config = spim::Config::default();
    config.mode = spim::MODE_3;
    config.orc = 122;
    let spim = spim::Spim::new_txonly(p.TWISPI0, Irqs, p.P0_14, p.P0_15, config);
    let spi_bus = NoopMutex::new(RefCell::new(spim));
    let spi_bus = SPI_BUS.init(spi_bus);
    let spi_dev: SpiDevice<'_, embassy_sync::blocking_mutex::raw::NoopRawMutex, Spim<'_, TWISPI0>, Output<'_>> = SpiDevice::new(spi_bus, cs);
    
    // tasks
    spawner.must_spawn(input::button_a_task(button_a));
    spawner.must_spawn(input::button_b_task(button_b));
    spawner.must_spawn(display::display_task(spi_dev, dc, rst));

    // heartbeat
    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        CHANNEL.send(true).await;
        ticker.next().await;
    }
}
