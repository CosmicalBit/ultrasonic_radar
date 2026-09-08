use std::{io, net::SocketAddr};

use raylib::prelude::*;
use tokio::net::{TcpListener, UdpSocket};
use ultrasonic_radar::protocol::SendHeader;

#[derive(Debug)]
pub enum NetworkError {
    BindError(io::Error),
    ReceiveError(io::Error),
    InvalidPacketLength(usize),
}

const ADDR: &str = " 300";

//TODO: hadle user input 
#[tokio::main]
async fn main() -> io::Result<()> {
    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);

        let position = Vector3::new(0.0, 0.0, 0.0);
        let direction = Vector3::new(0.0, 0.0, -1.0);

        let camera = Camera3D::perspective(position, position + direction, Vector3::new(0.0, 1.0, 0.0), 45.0);

        loop {
            let socket = UdpSocket::bind(ADDR).await?;

            let mut header = [0u8; 1];
            socket.recv_from(&mut header).await?;

            handle_header(&header, &socket, rl, &camera).await?;
        }
    }
    Ok(())
}

async fn handle_header(header: &[u8; 1], socket: &UdpSocket, rl: RaylibHandle, camera: &Camera3D) {
    let mut point = (SendHeader::Point as u8).to_be_bytes();

    match header[0] {
        point => {
            let point = read_point(socket).await?;
            rl.get_world_to_screen(point, camera);
        },
    }
}

async fn read_point(socket: &UdpSocket) -> io::Result<Vector3> {
    let mut x = [0u8; 4];
    let mut y = [0u8; 4];
    let mut z = [0u8; 4];

    socket.recv_from(&mut x).await?;
    socket.recv_from(&mut y).await?;
    socket.recv_from(&mut z).await?;

    Ok(Vector3 {
        x: f32::from_be_bytes(x),
        y: f32::from_be_bytes(y),
        z: f32::from_be_bytes(z),
    })
}
