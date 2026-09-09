use core::net::Ipv4Addr;

use embassy_executor::Spawner;
use embassy_net::{
    Config, DhcpConfig, Runner, StackResources,
    udp::{BindError, PacketMetadata, SendError, UdpSocket},
};
use embassy_time::Timer;
use esp_hal::rng::Rng;
use esp_radio::wifi::{Config as WifiConfig, Interface, Interfaces, WifiController, sta::StationConfig};
use static_cell::StaticCell;

const SSID: &str = env!("SSID", "set SSID in the project's .env file");
const PASSWORD: &str = match option_env!("PASSWORD") {
    Some(password) => password,
    None => "",
};
const DESTINY_IP: &str = env!("dest_ip", "set destiny ip in .env file");
const DESTINY_PORT: &str = env!("dest_port", "set destiny port in .env file");

macro_rules! mk_static {
    ($type:ty, $value:expr) => {{
        static STATIC_CELL: StaticCell<$type> = StaticCell::new();
        STATIC_CELL.init($value)
    }};
}

macro_rules! udp_socket {
    ($socket:ident, $stack:expr, $size:expr) => {
        let mut rx_meta = [PacketMetadata::EMPTY; 1];
        let mut rx_buffer = [0u8; $size];

        let mut tx_meta = [PacketMetadata::EMPTY; 1];
        let mut tx_buf = [0u8; $size];

        let mut $socket = UdpSocket::new($stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buf);
    };
}

pub(super) struct Disconnected {
    interface: Interfaces<'static>,
}

pub(super) struct Connected {
    stack: embassy_net::Stack<'static>,
}

pub(super) struct Wifi<State> {
    controller: WifiController<'static>,
    spawner: Spawner,
    state: State,
}

impl Wifi<Disconnected> {
    pub(super) const fn new(controller: WifiController<'static>, interface: Interfaces<'static>, spawner: Spawner) -> Self {
        Wifi {
            controller,
            spawner,
            state: Disconnected { interface },
        }
    }

    pub(super) async fn connect(mut self) -> Wifi<Connected> {
        let station_config = WifiConfig::Station(StationConfig::default().with_ssid(SSID).with_password(PASSWORD.into()));
        self.controller.set_config(&station_config).unwrap();

        loop {
            match self.controller.connect_async().await {
                Ok(_) => break,
                Err(error) => {
                    esp_println::println!("Wi-Fi connection failed: {error:?}; retrying in 5 seconds");
                    Timer::after_secs(5).await;
                },
            }
        }

        let net_config = Config::dhcpv4(DhcpConfig::default());

        let rng = Rng::new();
        let seed = (rng.random() as u64) << 32 | rng.random() as u64;

        let (stack, runner) = embassy_net::new(
            self.state.interface.station,
            net_config,
            mk_static!(StackResources<3>, StackResources::<3>::new()),
            seed,
        );

        self.spawner.spawn(net_task(runner).unwrap());
        stack.wait_config_up().await;

        Wifi {
            controller: self.controller,
            spawner: self.spawner,
            state: Connected { stack },
        }
    }
}
#[allow(dead_code)]
#[derive(Debug)]
pub enum NetworkError {
    BindError(BindError),
    SendError(SendError),
}

impl From<SendError> for NetworkError {
    fn from(value: SendError) -> Self {
        NetworkError::SendError(value)
    }
}

impl From<BindError> for NetworkError {
    fn from(value: BindError) -> Self {
        NetworkError::BindError(value)
    }
}

impl Wifi<Connected> {
    pub(super) async fn send_udp(&self, data: impl AsRef<[u8]>) -> Result<(), NetworkError> {
        udp_socket!(socket, self.state.stack, 400);

        let destiny_ip: Ipv4Addr = DESTINY_IP.parse().expect("invalid dest_ip in .env file");
        let destiny_port: u16 = DESTINY_PORT.parse().expect("invalid dest_port in .env file");

        socket.bind(random_udp_port())?;
        socket.send_to(data.as_ref(), (destiny_ip, destiny_port)).await?;

        Ok(())
    }
}

fn random_udp_port() -> u16 {
    let rng = Rng::new();
    49152 + (rng.random() as u16 % 16384)
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface<'static>>) {
    runner.run().await;
}
