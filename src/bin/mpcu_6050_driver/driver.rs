//! Register documentation: <https://dfimg.dfrobot.com/enshop/image/data/SEN0142/RM-MPU-6000A.pdf>

use core::marker::PhantomData;

use embedded_hal::i2c::I2c;
use esp_hal::i2c;

const WHO_AM_I: u8 = 0x75;
const DEVICE_ADDR: u8 = 0x68;
const PWR_MGMT_1: u8 = 0x6B;
const PWR_MGMT_2: u8 = 0x6C;
const FIFO_COUNT_H: u8 = 0x72;
const FIFO_COUNT_L: u8 = 0x73;
const MOT_THR: u8 = 0x1F;
const GYRO_CONFIG: u8 = 0x1B;

pub enum Error<E> {
    WhoAmI,
    I2c(E),
}

impl<E> From<E> for Error<E> {
    fn from(value: E) -> Self {
        Error::I2c(value)
    }
}
fn hello<BUS>(i2c: &mut BUS) -> Result<(), Error<BUS::Error>>
where
    BUS: embedded_hal::i2c::I2c,
{
    let mut red = [0u8; 1];
    i2c.write_read(DEVICE_ADDR, &[WHO_AM_I], &mut red)?;

    if red[0] != DEVICE_ADDR {
        return Err(Error::WhoAmI);
    }

    Ok(())
}
pub enum Capabilities {
    Accelerometer,
    Gyroscope,
}


pub struct OFF;
pub struct ON;
pub struct CapabilitiesSetted;

pub struct Mpu6050<BUS, State>
where
    BUS: embedded_hal::i2c::I2c,
{
    i2c: BUS,
    capabilities: Option<Capabilities>,
    _data: PhantomData<State>,
}

impl<BUS> Mpu6050<BUS, OFF>
where
    BUS: embedded_hal::i2c::I2c,
{
    pub fn init(mut i2c: BUS) -> Result<Self, Error<BUS::Error>> {
        hello(&mut i2c)?;

        Ok(Self {
            i2c,
            capabilities: None,
            _data: PhantomData,
        })
    }

    
    fn wake(&mut self) -> Result<(), Error<BUS::Error>> {
        let mut data = [0u8; 1];

        self.i2c.write_read(DEVICE_ADDR, &[PWR_MGMT_1], &mut data)?;

        data[0] &= !(1 << 6);
        self.i2c.write(DEVICE_ADDR, &[PWR_MGMT_1, data[0]])?;

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

    fn read_fifo_count(&mut self) -> Result<u16, Error<BUS::Error>> {
        let mut red = [0u8; 2];

        self.i2c.write_read(DEVICE_ADDR, &[FIFO_COUNT_H], &mut red)?;

        Ok(u16::from_be_bytes([red[0], red[1]]))
    }
}



impl<BUS: I2c> Mpu6050<BUS, OFF> {
    pub fn start(mut self) -> Result<Mpu6050<BUS, ON>, Error<BUS::Error>> {
        self.wake()?;

        Ok(Mpu6050::<BUS, ON> {
            i2c: self.i2c,
            capabilities: None,
            _data: PhantomData,
        })
    }
}

impl<BUS: I2c> Mpu6050<BUS, ON> {
    pub fn set_capabilities(self, capabilities: Capabilities) -> Mpu6050<BUS, CapabilitiesSetted> {
        Mpu6050::<BUS, CapabilitiesSetted> {
            _data: PhantomData,
            capabilities: Some(capabilities),
            i2c: self.i2c,
        }
    }
}
//after configure mesuraments ranges GYRO_CONFIG ACCEL_CONFIG
// TODO continue on page 44

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_wake_up_bit() {
        let resh = get_wake_up_frequency_bit(WakeUpFrequency::High);
        let resm = get_wake_up_frequency_bit(WakeUpFrequency::Medium);
        let resl = get_wake_up_frequency_bit(WakeUpFrequency::Low);
        let resvl = get_wake_up_frequency_bit(WakeUpFrequency::VeryLow);

        assert_eq!(resh, 3);
        assert_eq!(resm, 2);
        assert_eq!(resl, 1);
        assert_eq!(resvl, 0);
    }
}
