use rosc::{OscPacket, OscType};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Default)]
pub struct OscState {
    pub bass: f32,
    pub mid: f32,
    pub high: f32,
    pub bpm: f32,
    pub trigger: bool,
    pub custom: std::collections::HashMap<String, f32>,
}

pub type SharedOsc = Arc<Mutex<OscState>>;

pub struct OscHandler {
    socket: UdpSocket,
    shared: SharedOsc,
}

impl OscHandler {
    pub fn new(port: u16, shared: SharedOsc) -> anyhow::Result<Self> {
        let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        socket.set_read_timeout(Some(Duration::from_millis(10)))?;

        eprintln!("OSC listening on port {}", port);

        Ok(OscHandler { socket, shared })
    }

    pub fn recv(&self) {
        let mut buf = [0u8; 1024];

        while let Ok((size, _addr)) = self.socket.recv_from(&mut buf) {
            if let Ok((_, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                self.handle_packet(packet);
            }
        }
    }

    fn handle_packet(&self, packet: OscPacket) {
        let mut state = self.shared.lock().unwrap();

        match packet {
            OscPacket::Message(msg) => {
                let addr = msg.addr.clone();
                let args = &msg.args;

                match addr.as_str() {
                    "/bass" | "/audio/bass" => {
                        if let Some(OscType::Float(f)) = args.first() {
                            state.bass = *f;
                        }
                    }
                    "/mid" | "/audio/mid" => {
                        if let Some(OscType::Float(f)) = args.first() {
                            state.mid = *f;
                        }
                    }
                    "/high" | "/audio/high" => {
                        if let Some(OscType::Float(f)) = args.first() {
                            state.high = *f;
                        }
                    }
                    "/bpm" | "/tempo" => {
                        if let Some(OscType::Float(f)) = args.first() {
                            state.bpm = *f;
                        } else if let Some(OscType::Int(i)) = args.first() {
                            state.bpm = *i as f32;
                        }
                    }
                    "/trigger" | "/beat" => {
                        state.trigger = true;
                    }
                    _ => {
                        if let Some(OscType::Float(f)) = args.first() {
                            state.custom.insert(addr.clone(), *f);
                        }
                    }
                }
            }
            OscPacket::Bundle(_bundle) => {}
        }
    }

    pub fn clear_trigger(&self) {
        let mut state = self.shared.lock().unwrap();
        state.trigger = false;
    }
}

pub struct OscSender {
    socket: UdpSocket,
    target: SocketAddr,
}

impl OscSender {
    pub fn new(target_host: &str, target_port: u16) -> anyhow::Result<Self> {
        let target = SocketAddr::from((target_host.parse::<Ipv4Addr>()?, target_port));
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;

        eprintln!("OSC sending to {}:{}", target_host, target_port);

        Ok(OscSender { socket, target })
    }

    pub fn send_float(&self, addr: &str, value: f32) {
        let msg = rosc::OscMessage {
            addr: addr.to_string(),
            args: vec![OscType::Float(value)],
        };
        let packet = OscPacket::Message(msg);
        if let Ok(buf) = rosc::encoder::encode(&packet) {
            let _ = self.socket.send_to(&buf, self.target);
        }
    }

    #[allow(dead_code)]
    pub fn send_int(&self, addr: &str, value: i32) {
        let msg = rosc::OscMessage {
            addr: addr.to_string(),
            args: vec![OscType::Int(value)],
        };
        let packet = OscPacket::Message(msg);
        if let Ok(buf) = rosc::encoder::encode(&packet) {
            let _ = self.socket.send_to(&buf, self.target);
        }
    }
}

pub fn create_osc_receiver(port: u16) -> SharedOsc {
    let shared: SharedOsc = Arc::new(Mutex::new(OscState::default()));
    if let Ok(handler) = OscHandler::new(port, shared.clone()) {
        std::thread::spawn(move || loop {
            handler.recv();
            handler.clear_trigger();
            std::thread::sleep(Duration::from_millis(1));
        });
    }
    shared
}
