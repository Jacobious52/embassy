#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[cfg(feature = "defmt-rtt")]
use defmt_rtt::*;
use embassy_boot_stm32::{AlignedBuffer, FirmwareUpdater};
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::flash::{Flash, WRITE_SIZE};
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_time::{Duration, Timer};
use panic_reset as _;

static APP_B: &[u8] = include_bytes!("../../b.bin");

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let flash = Flash::unlock(p.FLASH);
    let mut flash = BlockingAsync::new(flash);

    let button = Input::new(p.PB2, Pull::Up);
    let mut button = ExtiInput::new(button, p.EXTI2);

    let mut led = Output::new(p.PB5, Level::Low, Speed::Low);

    led.set_high();

    let mut updater = FirmwareUpdater::default();
    button.wait_for_falling_edge().await;
    let mut offset = 0;
    for chunk in APP_B.chunks(128) {
        let mut buf: [u8; 128] = [0; 128];
        buf[..chunk.len()].copy_from_slice(chunk);
        updater.write_firmware(offset, &buf, &mut flash, 128).await.unwrap();
        offset += chunk.len();
    }

    let mut magic = AlignedBuffer([0; WRITE_SIZE]);
    updater.mark_updated(&mut flash, magic.as_mut()).await.unwrap();
    led.set_low();
    Timer::after(Duration::from_secs(1)).await;
    cortex_m::peripheral::SCB::sys_reset();
}
