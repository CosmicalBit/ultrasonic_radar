#![no_std]
#![no_main]

use esp_hal::{clock::CpuClock, delay::Delay, main};
use esp_println::println;
use esp32_dht11_rs::DHT11;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("panic: {info}");
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let polling_delay = Delay::new();
    let mut dht11 = DHT11::new(peripherals.GPIO1, Delay::new());

    println!("DHT11 GPIO1 diagnostic; waiting for sensor startup");
    polling_delay.delay_millis(2_000);

    loop {
        match dht11.read() {
            Ok(reading) => println!("DHT11 ok: {} C, {} %RH", reading.temperature, reading.humidity),
            Err(error) => println!("DHT11 error: {:?}", error),
        }
        polling_delay.delay_millis(2_000);
    }
}
