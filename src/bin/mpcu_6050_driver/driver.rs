//! Register documentation: <https://dfimg.dfrobot.com/enshop/image/data/SEN0142/RM-MPU-6000A.pdf>

use core::marker::PhantomData;

use embedded_hal::i2c::I2c;
use esp_hal::i2c;

const WHO_AM_I: u8 = 0x75;
const DEVICE_ADDR: u8 = 0x68;
const PWR_MGMT_1: u8 = 0x6B;
const FIFO_COUNT_H: u8 = 0x72;
const GYRO_CONFIG: u8 = 0x1B;
const ACCEL_CONFIG: u8 = 0x1C;

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
pub struct Capabilities {
    Accelerometer: bool,
    Gyroscope: bool,
}

pub struct OFF;
pub struct ON;
pub struct Configured;

pub struct Mpu6050<BUS, State>
where
    BUS: embedded_hal::i2c::I2c,
{
    i2c: BUS,
    _data: PhantomData<State>,
}

impl<BUS> Mpu6050<BUS, OFF>
where
    BUS: embedded_hal::i2c::I2c,
{
    pub fn init(mut i2c: BUS) -> Result<Self, Error<BUS::Error>> {
        hello(&mut i2c)?;

        Ok(Self { i2c, _data: PhantomData })
    }

    fn wake(&mut self) -> Result<(), Error<BUS::Error>> {
        let mut data = [0u8; 1];

        self.i2c.write_read(DEVICE_ADDR, &[PWR_MGMT_1], &mut data)?;

        data[0] &= !(1 << 6);
        self.i2c.write(DEVICE_ADDR, &[PWR_MGMT_1, data[0]])?;

        Ok(())
    }

    pub fn start(mut self) -> Result<Mpu6050<BUS, ON>, Error<BUS::Error>> {
        self.wake()?;

        Ok(Mpu6050::<BUS, ON> { i2c: self.i2c, _data: PhantomData })
    }
}

impl<BUS: I2c> Mpu6050<BUS, ON> {
    fn set_gyro_range(&mut self) -> Result<(), Error<BUS::Error>> {
        let mut data = [0u8; 1];

        self.i2c.write_read(DEVICE_ADDR, &[GYRO_CONFIG], &mut data)?;

        //clear both bits
        data[0] &= !(0b11 << 3);

        data[0] |= 1 << 3;
        data[0] |= 1 << 4;

        self.i2c.write(DEVICE_ADDR, &[GYRO_CONFIG, data[0]])?;

        Ok(())
    }
    fn accel_config(&mut self) -> Result<(), Error<BUS::Error>> {
        let mut data = [0u8; 1];

        self.i2c.write_read(DEVICE_ADDR, &[ACCEL_CONFIG], &mut data)?;

        data[0] &= !(1 << 4);
        data[0] &= !(1 << 3);

        data[0] |= 1 << 4;
        data[0] |= 1 << 3;

        self.i2c.write(DEVICE_ADDR, &[ACCEL_CONFIG, data[0]])?;

        Ok(())
    }

    pub fn config(mut self) -> Result<Mpu6050<BUS, Configured>, Error<BUS::Error>> {
        self.accel_config()?;
        self.set_gyro_range()?;

        Ok(Mpu6050::<BUS, Configured> { i2c: self.i2c, _data: PhantomData })
    }

    //this will be moooved to a new impl
    fn read_fifo_count(&mut self) -> Result<u16, Error<BUS::Error>> {
        let mut red = [0u8; 2];

        self.i2c.write_read(DEVICE_ADDR, &[FIFO_COUNT_H], &mut red)?;

        Ok(u16::from_be_bytes([red[0], red[1]]))
    }
}
