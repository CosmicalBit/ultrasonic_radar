use embedded_hal::i2c::I2c;
use esp_hal::{
    Blocking,
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    i2c::master::Config as I2cConfig,
    time::Instant,
};
type I2C_TYPE = esp_hal::i2c::master::I2c<'static, Blocking>;

use esp_println::println;
use esp32_dht11_rs::DHT11;
use ultrasonic_radar::mpu_6050_driver::driver::{Configured, Mpu6050};

pub struct Esp<BUS: I2c> {
    ultra_sonic_sensor: UltrasonicSensor,
    temp_sensor: DhtSensor,
    mpu_6050: Mpu6050<BUS, Configured>,
}

impl Esp<I2C_TYPE> {
    pub fn init() -> Self {
        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
        let peripherals = esp_hal::init(config);

        let trig = Output::new(peripherals.GPIO13, Level::Low, OutputConfig::default());
        let echo = Input::new(peripherals.GPIO14, InputConfig::default());

        let ultra_sonic_sensor = UltrasonicSensor::init(trig, echo);

        let delay = Delay::new();

        let dhh = DHT11::new(peripherals.GPIO1, delay);

        let dhh = DhtSensor::new(dhh);

        let i2c = I2C_TYPE::new(peripherals.I2C0, I2cConfig::default()).unwrap().with_sda(peripherals.GPIO5).with_scl(peripherals.GPIO3);

        let device = Mpu6050::init(i2c).unwrap();
        let device = device.start().unwrap();
        let device = device.config().unwrap();

        Self {
            ultra_sonic_sensor,
            temp_sensor: dhh,
            mpu_6050: device,
        }
    }
}

struct UltrasonicSensor {
    trig: Output<'static>,
    echo: Input<'static>,
}

impl UltrasonicSensor {
    const fn init(trig: Output<'static>, echo: Input<'static>) -> Self {
        Self { trig, echo }
    }
}

impl Esp<I2C_TYPE> {
    pub fn mesure_distance(&mut self) -> Option<f32> {
        //actual arguments
        let ultra_sensor = &mut self.ultra_sonic_sensor;
        let temp_sensor = &mut self.temp_sensor;

        let delay = Delay::new();
        ultra_sensor.trig.set_low();

        ultra_sensor.trig.set_high();
        delay.delay_micros(10);

        let echo_timeout = Instant::now();
        while ultra_sensor.echo.is_low() {
            if echo_timeout.elapsed().as_millis() >= 30 {
                return None;
            }
        }

        let start = Instant::now();

        while ultra_sensor.echo.is_high() {
            if start.elapsed().as_millis() >= 30 {
                return None;
            }
        }
        let high_time = start.elapsed();

        let (temp, _) = temp_sensor.mesure();

        let speed_sound_cm_per_us = (331.3 + 0.606 * temp as f32) / 10_000.0;

        println!("{speed_sound_cm_per_us}");

        //distance calculation following a existing knowed formula
        Some(high_time.as_micros() as f32 * speed_sound_cm_per_us / 2.0)
    }
    /// Returns the accelerometer and gyroscope values from the same FIFO sample.
    pub fn mesure_motion(&mut self) -> Option<((i16, i16, i16), (i16, i16, i16))> {
        let data = self.mpu_6050.read_data().unwrap()?;

        Some(((data.accel_x, data.accel_y, data.accel_z), (data.gyro_x, data.gyro_y, data.gyro_z)))
    }
}

struct DhtSensor(DHT11<'static, Delay>);

impl DhtSensor {
    const fn new(dhh: DHT11<'static, Delay>) -> Self {
        Self(dhh)
    }

    ///returns (temp, hummidity)
    fn mesure(&mut self) -> (i8, u8) {
        println!("befor read inside");
        let read = self.0.read();

        println!("read returned");

        match read {
            Ok(read) => {
                println!("DHT good");
                (read.temperature, read.humidity)
            },
            Err(err) => {
                println!("dht err {:?}", err);
                (0, 0)
            },
        }
    }
}
