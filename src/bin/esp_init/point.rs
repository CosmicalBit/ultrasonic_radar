use esp_hal::{Blocking, i2c::master::I2c};

use ultrasonic_radar::protocol::SendHeader;

use crate::esp_init::{esp::Esp, wifi::NetworkError};

pub struct Point((f32, f32, f32));

impl Point {
    pub  const fn new(point: (f32, f32, f32)) -> Self {
        Self(point)
    }
}

impl NetSend for Point {
    async fn send_udp(self, esp: &Esp<I2c<'static, Blocking>>) -> Result<(), NetworkError> {
        let (x, y, z) = self.0;

        let (x, y, z) = (x.to_be_bytes(), y.to_be_bytes(), z.to_be_bytes());

        let header = SendHeader::Point.to_bytes();

        esp.send_udp_internal(header).await?;
        esp.send_udp_internal(x).await?;
        esp.send_udp_internal(y).await?;
        esp.send_udp_internal(z).await?;

        Ok(())
    }
}

pub trait NetSend {
    async fn send_udp(self, esp: &Esp<I2c<'static, Blocking>>) -> Result<(), NetworkError>;
}
