#![no_std]

use embedded_hal::delay::DelayNs;
use esp_hal::{
    gpio::{DriveMode, Flex, InputConfig, OutputConfig, Pin},
    time::Instant,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SensorError {
    BusLow,
    ResponseStartTimeout,
    ResponseLowTimeout,
    ResponseHighTimeout,
    BitLowTimeout { bit: u8 },
    BitHighTimeout { bit: u8 },
    ChecksumMismatch { expected: u8, actual: u8 },
}

#[derive(Debug, Copy, Clone)]
pub struct Reading {
    pub humidity: u8,
    pub temperature: i8,
}

pub struct DHT11<'a, D> {
    pub pin: Flex<'a>,
    pub delay: D,
}

const RESPONSE_TIMEOUT_US: u64 = 1_000;
const BIT_TIMEOUT_US: u64 = 200;
const BIT_SAMPLE_US: u32 = 35;

impl<'a, D> DHT11<'a, D>
where
    D: DelayNs,
{
    pub fn new(pin: impl Pin + 'a, delay: D) -> Self {
        let mut pin = Flex::new(pin);
        let out_config = OutputConfig::default().with_drive_mode(DriveMode::OpenDrain);
        pin.apply_output_config(&out_config);
        pin.apply_input_config(&InputConfig::default());
        pin.set_high();
        pin.set_input_enable(true);
        pin.set_output_enable(false);
        Self { pin, delay }
    }

    pub fn read(&mut self) -> Result<Reading, SensorError> {
        let data = self.read_raw()?;
        let (negative, magnitude) = convert_signed(data[2]);
        let temperature = if negative { -(magnitude as i8) } else { magnitude as i8 };

        Ok(Reading { temperature, humidity: data[0] })
    }

    fn read_raw(&mut self) -> Result<[u8; 5], SensorError> {
        // Start from a released bus and verify that the external pull-up works.
        self.pin.set_high();
        self.pin.set_input_enable(true);
        self.pin.set_output_enable(false);
        self.delay.delay_us(10);
        if self.pin.is_low() {
            return Err(SensorError::BusLow);
        }

        // The output latch is already low when output is enabled, avoiding a
        // push-pull/high glitch before the DHT11 start pulse.
        self.pin.set_low();
        self.pin.set_output_enable(true);
        self.delay.delay_ms(20);

        // In open-drain mode, HIGH releases the line. Keep the input buffer
        // enabled, then fully disable output after the DHT11 release period.
        self.pin.set_high();
        self.delay.delay_us(30);
        self.pin.set_output_enable(false);

        // DHT11 response: about 80 us LOW, about 80 us HIGH, then the first
        // data bit begins with a LOW preamble. Synchronize on all three edges.
        self.wait_while_high(RESPONSE_TIMEOUT_US, SensorError::ResponseStartTimeout)?;
        self.wait_while_low(RESPONSE_TIMEOUT_US, SensorError::ResponseLowTimeout)?;
        self.wait_while_high(RESPONSE_TIMEOUT_US, SensorError::ResponseHighTimeout)?;

        let mut data = [0; 5];
        for (byte_index, byte) in data.iter_mut().enumerate() {
            *byte = self.read_byte((byte_index * 8) as u8)?;
        }

        let checksum = data[0].wrapping_add(data[1]).wrapping_add(data[2]).wrapping_add(data[3]);

        if data[4] == checksum {
            Ok(data)
        } else {
            Err(SensorError::ChecksumMismatch { expected: checksum, actual: data[4] })
        }
    }

    fn read_byte(&mut self, first_bit: u8) -> Result<u8, SensorError> {
        let mut byte = 0;
        for bit in 0..8u8 {
            let absolute_bit = first_bit + bit;
            self.wait_while_low(BIT_TIMEOUT_US, SensorError::BitLowTimeout { bit: absolute_bit })?;

            self.delay.delay_us(BIT_SAMPLE_US);
            if self.pin.is_high() {
                byte |= 1 << (7 - bit);
            }

            self.wait_while_high(BIT_TIMEOUT_US, SensorError::BitHighTimeout { bit: absolute_bit })?;
        }
        Ok(byte)
    }

    fn wait_while_low(&self, timeout_us: u64, error: SensorError) -> Result<(), SensorError> {
        let wait_started = Instant::now();
        while self.pin.is_low() {
            if wait_started.elapsed().as_micros() >= timeout_us {
                return Err(error);
            }
        }
        Ok(())
    }

    fn wait_while_high(&self, timeout_us: u64, error: SensorError) -> Result<(), SensorError> {
        let wait_started = Instant::now();
        while self.pin.is_high() {
            if wait_started.elapsed().as_micros() >= timeout_us {
                return Err(error);
            }
        }
        Ok(())
    }
}

fn convert_signed(value: u8) -> (bool, u8) {
    (value & 0x80 != 0, value & 0x7f)
}
