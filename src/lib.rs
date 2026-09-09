#![no_std]

#[cfg(target_arch = "xtensa")]
pub mod mpu_6050_driver;
pub mod protocol;
