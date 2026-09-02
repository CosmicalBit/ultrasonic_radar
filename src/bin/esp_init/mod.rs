use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    time::Instant,
};
use esp_println::println;
use esp32_dht11_rs::DHT11;

pub struct Esp {
    ultra_sonic_sensor: UltrasonicSensor,
    temp_sensor: DhtSensor,
}

pub trait Init {
    fn init() -> Self;
}

impl Init for Esp {
    fn init() -> Self {
        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
        let peripherals = esp_hal::init(config);

        let trig = Output::new(peripherals.GPIO13, Level::Low, OutputConfig::default());
        let echo = Input::new(peripherals.GPIO14, InputConfig::default());

        let ultra_sonic_sensor = UltrasonicSensor::init(trig, echo);

        let delay = Delay::new();

        let dhh = DHT11::new(peripherals.GPIO1, delay);

        let dhh = DhtSensor::new(dhh);

        Self { ultra_sonic_sensor, temp_sensor: dhh }
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

impl Esp {
    pub fn mesure_distance(&mut self) -> f32 {
        //actual arguments
        let ultra_sensor = &mut self.ultra_sonic_sensor;
        let temp_sensor = &mut self.temp_sensor;

        let delay = Delay::new();
        ultra_sensor.trig.set_low();

        ultra_sensor.trig.set_high();
        delay.delay_micros(10);

        while ultra_sensor.echo.is_low() {}

        let start = Instant::now();

        while ultra_sensor.echo.is_high() {}
        let high_time = start.elapsed();

        println!("before temp");
        let (temp, _) = temp_sensor.mesure();

        println!("{temp}");
        let speed_sound_cm_per_us = (331.3 + 0.606 * temp as f32) / 10_000.0;

        println!("{speed_sound_cm_per_us}");

        //distance calculation following a existing knowed formula
        high_time.as_micros() as f32 * speed_sound_cm_per_us / 2.0
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

