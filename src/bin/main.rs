#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::main;
use esp_println::println;

use crate::esp_init::{Esp, Init};
mod esp_init;

#[panic_handler]
pub fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        println!("panic panic")
    }
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(clippy::large_stack_frames, reason = "it's not unusual to allocate larger buffers etc. in main")]
#[main]
fn main() -> ! {
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);

    let mut esp = Esp::init();
    println!("inited");
    loop {
        let distance = esp.mesure_distance();
        println!("the object distance is {distance} cm ");
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}
