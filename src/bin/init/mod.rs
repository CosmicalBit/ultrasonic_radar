use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    time::Instant,
};
use esp32_dht11_rs::DHT11;

struct Esp {
    ultra_sonic_sensor: UltrasonicSensor,
    dht11: DHT11<'static, Delay>,
}

pub trait Init {
    type Input;
    fn init(input: &Self::Input) -> Self;
}

impl Init for Esp {
    type Input = ();
    fn init(_input: &Self::Input) -> Self {
        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
        let peripherals = esp_hal::init(config);

        let trig = Output::new(peripherals.GPIO13, Level::Low, OutputConfig::default());
        let echo = Input::new(peripherals.GPIO14, InputConfig::default());

        let ultra_sonic_sensor = UltrasonicSensor::init(trig, echo);

        let delay = Delay::new();
        let dht11 = DHT11::new(peripherals.GPIO11, delay);
        Self {
            ultra_sonic_sensor,
            dht11,
        }
    }
}

struct UltrasonicSensor {
    trig: Output<'static>,
    echo: Input<'static>,
}

impl UltrasonicSensor {
    fn init(trig: Output<'static>, echo: Input<'static>) -> Self {
        Self { trig, echo }
    }
}

const SPEED_OF_SOUND: f32 = 0.034342;

impl Esp {
    fn mesure(sensor: &mut UltrasonicSensor) {
        let delay = Delay::new();
        sensor.trig.set_low();

        sensor.trig.set_high();
        delay.delay_micros(10);

        while sensor.echo.is_low() {}

        let start = Instant::now();

        while sensor.echo.is_high() {}
        let _high_time = start.elapsed();
    }
}
