use core::ptr::read;

use embedded_hal::i2c::I2c;
use esp_hal::{
    i2c::{self, master::Config},
    time::Rate,
};

const CONFIG: u8 = 0x1A;
const WHO_AM_I: u8 = 0x75;
const DEVICE_ADDR: u8 = 0x68;

fn write_config<I2C>(i2c: &mut I2C, value: u8) -> Result<(), I2C::Error>
where
    I2C: embedded_hal::i2c::I2c,
{
    unsafe { i2c.write(0x68, &[CONFIG, value]) }
}

enum Error {
    WhoAmI,
    I2C,
}

fn hello<I2C>(i2c: &mut I2C) -> Result<(), Error>
where
    I2C: embedded_hal::i2c::I2c,
{
    let mut red = [0u8; 1];
    unsafe { i2c.write_read(DEVICE_ADDR, &[WHO_AM_I], &mut red).map_err(|_| Error::WhoAmI) }?;

    if red[0] != DEVICE_ADDR {
        return Err(Error::WhoAmI);
    }

    Ok(())
}

struct MPU_6050<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    i2c: I2C,
}

impl<I2C> MPU_6050<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    fn init(mut i2c: I2C) -> Result<Self, Error> {
        hello(&mut i2c)?;

        Ok(Self { i2c })
    }
}
