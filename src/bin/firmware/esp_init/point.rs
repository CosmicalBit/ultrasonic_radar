use esp_hal::{Blocking, i2c::master::I2c};
use ultrasonic_radar::protocol::SendHeader;

use crate::esp_init::{esp::Esp, wifi::NetworkError};

pub struct Point((f32, f32, f32));

impl Point {
    pub const fn new(point: (f32, f32, f32)) -> Self {
        Self(point)
    }
}
impl Point {
    fn to_packet(self) -> [u8; 13] {
        let x = self.0.0.to_be_bytes();
        let y = self.0.1.to_be_bytes();
        let z = self.0.2.to_be_bytes();

        // 13 is the sum of x + y + z ammount of bytes plus header
        let mut bytes = [0u8; 13];

        bytes[0] = SendHeader::Point as u8;
        bytes[1..5].copy_from_slice(&x);
        bytes[5..9].copy_from_slice(&y);
        bytes[9..13].copy_from_slice(&z);

        bytes
    }
}
impl NetSend for Point {
    async fn send_udp(self, esp: &Esp<I2c<'static, Blocking>>) -> Result<(), NetworkError> {
        let packet = self.to_packet();

        esp.send_udp_internal(packet).await?;

        Ok(())
    }
}

pub trait NetSend {
    async fn send_udp(self, esp: &Esp<I2c<'static, Blocking>>) -> Result<(), NetworkError>;
}
