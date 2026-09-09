use esp_hal::{Blocking, i2c::master::I2c};
use ultrasonic_radar::protocol::SendHeader;

use crate::esp_init::{esp::Esp, wifi::NetworkError};

pub struct Point((f32, f32, f32));

impl Point {
    pub const fn new(point: (f32, f32, f32)) -> Self {
        Self(point)
    }
}

impl NetSend for Point {
    async fn send_udp(self, esp: &Esp<I2c<'static, Blocking>>) -> Result<(), NetworkError> {
        let (x, y, z) = self.0;

        let mut packet = [0_u8; 13];
        packet[0] = SendHeader::Point as u8;
        packet[1..5].copy_from_slice(&x.to_be_bytes());
        packet[5..9].copy_from_slice(&y.to_be_bytes());
        packet[9..13].copy_from_slice(&z.to_be_bytes());

        esp.send_udp_internal(packet).await?;

        Ok(())
    }
}

pub trait NetSend {
    async fn send_udp(self, esp: &Esp<I2c<'static, Blocking>>) -> Result<(), NetworkError>;
}
