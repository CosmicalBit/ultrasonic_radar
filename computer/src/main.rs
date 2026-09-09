use std::io;

use raylib::prelude::*;
use tokio::net::UdpSocket;
use ultrasonic_radar::protocol::SendHeader;

const ADDR: &str = "0.0.0.0:5000";

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = UdpSocket::bind(ADDR).await?;
    let (mut rl, thread) = raylib::init().size(640, 480).title("Ultrasonic radar").build();
    rl.set_target_fps(60);

    let position = Vector3::new(0.0, 0.0, 0.0);
    let direction = Vector3::new(0.0, 0.0, -1.0);
    let camera = Camera3D::perspective(position, position + direction, Vector3::new(0.0, 1.0, 0.0), 45.0);
    let mut points = Vec::new();
    let mut packet = [0_u8; 13];

    while !rl.window_should_close() {
        loop {
            match socket.try_recv_from(&mut packet) {
                Ok((length, _)) => handle_packet(&packet[..length], &mut points),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) => return Err(error),
            }
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        for &point in &points {
            let screen_point = d.get_world_to_screen(point, camera);
            d.draw_circle_v(screen_point, 4.0, Color::RED);
        }

        d.draw_text(&format!("Points: {}", points.len()), 10, 10, 20, Color::DARKGRAY);
    }

    Ok(())
}

fn handle_packet(packet: &[u8], points: &mut Vec<Vector3>) {
    if packet.first() == Some(&(SendHeader::Point as u8)) {
        if let Some(point) = read_point(&packet[1..]) {
            points.push(point);
        }
    }
}

fn read_point(bytes: &[u8]) -> Option<Vector3> {
    if bytes.len() != 12 {
        return None;
    }

    Some(Vector3 {
        x: f32::from_be_bytes(bytes[0..4].try_into().ok()?),
        y: f32::from_be_bytes(bytes[4..8].try_into().ok()?),
        z: f32::from_be_bytes(bytes[8..12].try_into().ok()?),
    })
}
