//! Register documentation: <https://dfimg.dfrobot.com/enshop/image/data/SEN0142/RM-MPU-6000A.pdf>

use core::{fmt::Debug, marker::PhantomData};

use embedded_hal::i2c::I2c;

const WHO_AM_I: u8 = 0x75;
const DEVICE_IDS: [u8; 2] = [0x68, 0x70];
const DEVICE_ADDRESSES: [u8; 2] = [0x68, 0x69];
const PWR_MGMT_1: u8 = 0x6B;
const FIFO_EN: u8 = 0x23;
const USER_CTRL: u8 = 0x6A;
const FIFO_COUNT_H: u8 = 0x72;
const GYRO_CONFIG: u8 = 0x1B;
const ACCEL_CONFIG: u8 = 0x1C;
const FIFO_R_W: u8 = 0x74;

//bcs is gyro + accel
const SAMPLE_SIZE: usize = 12;
#[derive(Debug)]
pub enum Error<E> {
    WhoAmI(u8),
    I2c(E),
}

impl<E> From<E> for Error<E> {
    fn from(value: E) -> Self {
        Error::I2c(value)
    }
}

fn hello<BUS>(i2c: &mut BUS, address: u8) -> Result<(), Error<BUS::Error>>
where
    BUS: embedded_hal::i2c::I2c,
{
    let mut red = [0u8; 1];
    i2c.write_read(address, &[WHO_AM_I], &mut red)?;

    if !DEVICE_IDS.contains(&red[0]) {
        return Err(Error::WhoAmI(red[0]));
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
    address: u8,
    _data: PhantomData<State>,
}

impl<BUS> Mpu6050<BUS, OFF>
where
    BUS: embedded_hal::i2c::I2c,
{
    pub fn init(mut i2c: BUS) -> Result<Self, Error<BUS::Error>> {
        let address = match hello(&mut i2c, DEVICE_ADDRESSES[0]) {
            Ok(()) => DEVICE_ADDRESSES[0],
            Err(Error::I2c(_)) => {
                hello(&mut i2c, DEVICE_ADDRESSES[1])?;
                DEVICE_ADDRESSES[1]
            },
            Err(error) => return Err(error),
        };

        Ok(Self {
            i2c,
            address,
            _data: PhantomData,
        })
    }

    fn wake(&mut self) -> Result<(), Error<BUS::Error>> {
        let mut data = [0u8; 1];

        self.i2c.write_read(self.address, &[PWR_MGMT_1], &mut data)?;

        data[0] &= !(1 << 6);
        self.i2c.write(self.address, &[PWR_MGMT_1, data[0]])?;

        Ok(())
    }

    pub fn start(mut self) -> Result<Mpu6050<BUS, ON>, Error<BUS::Error>> {
        self.wake()?;

        Ok(Mpu6050::<BUS, ON> {
            i2c: self.i2c,
            address: self.address,
            _data: PhantomData,
        })
    }
}

impl<BUS: I2c> Mpu6050<BUS, ON> {
    fn set_gyro_range(&mut self) -> Result<(), Error<BUS::Error>> {
        let mut data = [0u8; 1];

        self.i2c.write_read(self.address, &[GYRO_CONFIG], &mut data)?;

        //clear both bits
        data[0] &= !(0b11 << 3);

        data[0] |= 1 << 3;
        data[0] |= 1 << 4;

        self.i2c.write(self.address, &[GYRO_CONFIG, data[0]])?;

        Ok(())
    }
    fn accel_config(&mut self) -> Result<(), Error<BUS::Error>> {
        let mut data = [0u8; 1];

        self.i2c.write_read(self.address, &[ACCEL_CONFIG], &mut data)?;

        data[0] &= !(1 << 4);
        data[0] &= !(1 << 3);

        data[0] |= 1 << 4;
        data[0] |= 1 << 3;

        self.i2c.write(self.address, &[ACCEL_CONFIG, data[0]])?;

        Ok(())
    }

    fn enable_fifo(&mut self) -> Result<(), Error<BUS::Error>> {
        // Store accelerometer and all three gyroscope axes in the FIFO.
        self.i2c.write(self.address, &[FIFO_EN, 0b0111_1000])?;

        let mut user_ctrl = [0u8; 1];
        self.i2c.write_read(self.address, &[USER_CTRL], &mut user_ctrl)?;
        user_ctrl[0] |= 1 << 6;
        self.i2c.write(self.address, &[USER_CTRL, user_ctrl[0]])?;

        Ok(())
    }

    pub fn config(mut self) -> Result<Mpu6050<BUS, Configured>, Error<BUS::Error>> {
        self.accel_config()?;
        self.set_gyro_range()?;
        self.enable_fifo()?;

        Ok(Mpu6050::<BUS, Configured> {
            i2c: self.i2c,
            address: self.address,
            _data: PhantomData,
        })
    }
}

pub struct SampleData {
    pub accel_x: i16,
    pub accel_y: i16,
    pub accel_z: i16,
    pub gyro_x: i16,
    pub gyro_y: i16,
    pub gyro_z: i16,
}

impl<BUS: I2c> Mpu6050<BUS, Configured> {
    fn read_sample(&mut self) -> Result<[i16; 6], BUS::Error> {
        let mut data = [0u8; SAMPLE_SIZE];

        self.i2c.write_read(self.address, &[FIFO_R_W], &mut data)?;

        Ok([
            i16::from_be_bytes([data[0], data[1]]),   // accel X
            i16::from_be_bytes([data[2], data[3]]),   // accel Y
            i16::from_be_bytes([data[4], data[5]]),   // accel Z
            i16::from_be_bytes([data[6], data[7]]),   // gyro X
            i16::from_be_bytes([data[8], data[9]]),   // gyro Y
            i16::from_be_bytes([data[10], data[11]]), // gyro Z
        ])
    }
    fn read_fifo_count(&mut self) -> Result<u16, Error<BUS::Error>> {
        let mut red = [0u8; 2];

        self.i2c.write_read(self.address, &[FIFO_COUNT_H], &mut red)?;

        Ok(u16::from_be_bytes([red[0], red[1]]))
    }
    pub fn read_data(&mut self) -> Result<Option<SampleData>, Error<BUS::Error>> {
        let bytes = self.read_fifo_count()?;

        if bytes < SAMPLE_SIZE as u16 {
            return Ok(None);
        }

        let sample = self.read_sample()?;

        Ok(Some(SampleData {
            accel_x: sample[0],
            accel_y: sample[1],
            accel_z: sample[2],
            gyro_x: sample[3],
            gyro_y: sample[4],
            gyro_z: sample[5],
        }))
    }
}
