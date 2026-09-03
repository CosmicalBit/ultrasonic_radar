use libm::{atan2f, cosf, sinf, sqrtf};
use core::prelude::rust_2024::derive;
use core::default::Default;
use core::fmt::Debug;
use core::clone::Clone;

#[derive(Debug, Clone, Default)]
pub struct Orientation {
    roll: f32,
    pitch: f32,
    yaw: f32,
}

pub fn update_orientation(
    mut orientation: Orientation,
    accel: (i16, i16, i16),
    gyro: (i16, i16, i16),
    dt: f32,
) -> Orientation {
    let ax = accel.0 as f32;
    let ay = accel.1 as f32;
    let az = accel.2 as f32;

    // Orientação estimada pela gravidade
    let roll_acc = atan2f(ay, az);
    let pitch_acc = atan2f(-ax, sqrtf(ay * ay + az * az));

    // Se estiveres a usar ±2000 °/s no MPU6050:
    const GYRO_SCALE: f32 = 16.4;

    // Converter raw -> graus/s -> radianos/s
    let gx = (gyro.0 as f32 / GYRO_SCALE).to_radians();
    let gy = (gyro.1 as f32 / GYRO_SCALE).to_radians();
    let gz = (gyro.2 as f32 / GYRO_SCALE).to_radians();

    // Filtro complementar
    const ALPHA: f32 = 0.98;

    orientation.roll = ALPHA * (orientation.roll + gx * dt) + (1.0 - ALPHA) * roll_acc;

    orientation.pitch = ALPHA * (orientation.pitch + gy * dt) + (1.0 - ALPHA) * pitch_acc;

    // MPU6050 não tem magnetómetro:
    // yaw só pode ser integrado e vai acumular drift.
    orientation.yaw += gz * dt;

    orientation
}
pub fn calculate_pointed_point(position: (f32, f32, f32), orientation: Orientation, distance: f32) -> (f32, f32, f32) {
    let pitch = orientation.pitch;
    let yaw = orientation.yaw;

    // Vetor unitário da direção
    let dx = cosf(pitch) * cosf(yaw);
    let dy = cosf(pitch) * sinf(yaw);
    let dz = sinf(pitch);

    (position.0 + distance * dx, position.1 + distance * dy, position.2 + distance * dz)
}
