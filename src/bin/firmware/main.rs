#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use esp_hal::{main, time::Instant};
use esp_println::println;

use crate::{
    esp_init::{
        esp::Esp,
        point::{NetSend, Point},
    },
    position::{Orientation, calculate_pointed_point, update_orientation},
};

mod debug;
mod esp_init;
mod position;

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("panic: {info}");
    loop {
        core::hint::spin_loop();
    }
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();
#[allow(clippy::large_stack_frames, reason = "it's not unusual to allocate larger buffers etc. in main")]
#[esp_rtos::main]
async fn main(spwaner: Spawner) -> ! {
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);

    let mut esp = Esp::init(spwaner).await;
    let mut orientation = Orientation::default();
    let mut previous_sample = Instant::now();
    println!("inited");

    loop {
        let (accel, gyro) = match esp.mesure_motion() {
            Some(sample) => sample,
            None => continue,
        };

        let now = Instant::now();
        let dt = (now - previous_sample).as_micros() as f32 / 1_000_000.0;
        previous_sample = now;
        orientation = update_orientation(orientation, accel, gyro, dt);

        let distance = match esp.mesure_distance() {
            Some(distance) => distance,
            None => continue,
        };

        let pointed_point = calculate_pointed_point((0.0, 0.0, 0.0), orientation.clone(), distance);

        println!("distance: {distance} cm, pointed point: {:?}", pointed_point);

        let pointed_point = Point::new(pointed_point);
        pointed_point.send_udp(&esp).await.unwrap();
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}
