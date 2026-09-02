use core::{from, ptr::read};

use embedded_hal::i2c::I2c;
use esp_hal::{
    i2c::{self, master::Config},
    time::Rate,
};

const CONFIG: u8 = 0x1A;
const WHO_AM_I: u8 = 0x75;
const DEVICE_ADDR: u8 = 0x68;
const PWR_MGMT_1: u8 = 0x6B;
const PWR_MGMT_2: u8 = 0x6C;

enum Error<E> {
    WhoAmI,
    I2C(E),
}
impl<E> From<E> for Error<E> {
    fn from(value: E) -> Self {
        Error::I2C(value)
    }
}
fn hello<BUS>(i2c: &mut BUS) -> Result<(), Error<BUS::Error>>
where
    BUS: embedded_hal::i2c::I2c,
{
    let mut red = [0u8; 1];
    unsafe { i2c.write_read(DEVICE_ADDR, &[WHO_AM_I], &mut red)? };

    if red[0] != DEVICE_ADDR {
        return Err(Error::WhoAmI);
    }

    Ok(())
}

pub enum WakeUpFrequency {
    VeryLow,
    Low,
    Medium,
    High,
}

struct MPU_6050<BUS>
where
    BUS: embedded_hal::i2c::I2c,
{
    i2c: BUS,
}

impl<BUS> MPU_6050<BUS>
where
    BUS: embedded_hal::i2c::I2c,
{
    fn init(mut i2c: BUS) -> Result<Self, Error<BUS::Error>> {
        hello(&mut i2c)?;

        Ok(Self { i2c })
    }

    ///This register allows the user to configure the frequency of wake-ups in Accelerometer Only Low
    /// Power Mode. This register also allows the user to put individual axes of the accelerometer and
    ///gyroscope into standby mode.
    ///
    ///Description:
    ///The MPU-60X0 can be put into Accelerometer Only Low Power Mode using the following steps:
    ///     (i) Set CYCLE bit to 1
    ///     (ii) Set SLEEP bit to 0
    ///     (iii) Set TEMP_DIS bit to 1
    ///     (iv) Set STBY_XG, STBY_YG, STBY_ZG bits to 1
    fn leave_sleep_mode(&mut self) -> Result<(), Error<BUS::Error>> {
        let mut red = [0u8, 1];
        self.i2c.write_read(DEVICE_ADDR, &[PWR_MGMT_1], &mut red)?;

        let mut byte = red[0];

        //disable sleep
        byte &= !(1 << 6);
        //enable cycle mode
        byte |= 1 << 5;

        //set TEMP_DIS bit to one
        byte |= 1 << 3;
        self.i2c.write(DEVICE_ADDR, &[PWR_MGMT_1, byte])?;

        self.i2c.write_read(DEVICE_ADDR, &[PWR_MGMT_2], &mut red)?;

        let mut byte = red[0];

        byte |= 1 << 2;
        byte |= 1 << 1;
        byte |= 1 << 0 ;

        self.i2c.write(DEVICE_ADDR, &[PWR_MGMT_2, byte])?;

        Ok(())
    }
    /// LP_WAKE_CTRL    Wake-up Frequency
    /// 0               1.25 Hz
    /// 1               5 Hz
    /// 2               20 Hz
    /// 3               40 Hz
    fn set_wake_up_frequency(&mut self, frequency: WakeUpFrequency) -> Result<(), Error<BUS::Error>> {
        let frequency = get_wake_up_frequency_bit(frequency);

        let mut red = [0u8; 1];
        self.i2c.write_read(DEVICE_ADDR, &[PWR_MGMT_2], &mut red)?;

        let mut byte = red[0];
        
        byte &= !(0b11 << 6);
        byte |= frequency << 6;
        
        self.i2c.write(DEVICE_ADDR, &[PWR_MGMT_2, byte])?;

        Ok(())
    }
}

/// LP_WAKE_CTRL    Wake-up Frequency
/// 0               1.25 Hz
/// 1               5 Hz
/// 2               20 Hz
/// 3               40 Hz
fn get_wake_up_frequency_bit(frequency: WakeUpFrequency) -> u8 {
    match frequency {
        WakeUpFrequency::VeryLow => 0,
        WakeUpFrequency::Low => 1,
        WakeUpFrequency::Medium => 2,
        WakeUpFrequency::High => 3,
    }
}
// TODO continue on page 44