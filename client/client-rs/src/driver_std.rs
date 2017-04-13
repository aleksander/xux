use std;
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::io::Read;
use std::io::Write;

use driver::{Driver, Event};

use errors::*;

impl Driver for DriverStd {
    // fn new (ip: std::net::IpAddr, port: u16) -> Result<&'a Driver>;
    fn tx(&self, buf: &[u8]) -> Result<()> {
        self.tx(buf)
    }
    fn timeout(&self, seq: usize, ms: u64) {
        self.timeout(seq, ms);
    }
    fn event(&mut self) -> Result<Event> {
        self.event()
    }
}

pub struct DriverStd {
    sock: std::net::UdpSocket,
    tx: Sender<Event>, // XXX maybe use SyncSender ?
    rx: Receiver<Event>,
}

impl DriverStd {
    pub fn new(host: &str, port: u16) -> Result<DriverStd> {
        let sock = std::net::UdpSocket::bind("0.0.0.0:0").chain_err(||"driverstd.new bind")?;
        let (tx, rx) = channel();
        sock.connect((host, port)).chain_err(||"udp_sock::connect")?;

        let receiver_tx = tx.clone();
        let sock_rx = sock.try_clone().chain_err(||"driver::new.try_clone(sock)")?;
        thread::spawn(move || {
            let mut buf = vec![0; 65535];
            loop {
                let len = sock_rx.recv(&mut buf).expect("driver::recv");
                // TODO zero-copy data processing
                receiver_tx.send(Event::Rx(buf[..len].to_vec())).expect("driver::send(event::rx)");
            }
        });

        let render_tx = tx.clone();
        thread::spawn(move || {
            let listener = std::net::TcpListener::bind("127.0.0.1:8080").expect("driver::new.tcp_new");
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let _tx = render_tx.clone();
                        thread::spawn(move || {
                            let mut buf = vec![0; 1024];
                            let (reply_tx, reply_rx) = channel();
                            loop {
                                let len = stream.read(&mut buf).expect("driver::tcp_stream.read");
                                _tx.send(Event::Tcp((reply_tx.clone(), buf[..len].to_vec())))
                                    .expect("driver::send(event::tcp)");
                                let reply = reply_rx.recv().expect("driver::recv_rx");
                                // info!("RENDERRED REPLY: {:?}", reply);
                                let _len = stream.write(reply.as_bytes()).expect("strem.write");
                            }
                        });
                    }
                    Err(e) => {
                        info!("connection error: {:?}", e);
                        break;
                    }
                }
            }
        });

        Ok(DriverStd {
            sock: sock,
            tx: tx,
            rx: rx,
        })
    }

    pub fn tx(&self, buf: &[u8]) -> Result<()> {
        // info!("driver.tx: {} bytes", buf.len());
        let len = self.sock.send(buf).chain_err(||"driver.tx")?;
        if len != buf.len() {
            return Err("sent len != buf len".into());
        }
        Ok(())
    }

    pub fn timeout(&self, seq: usize, ms: u64) {
        use std::time::Duration;

        // info!("driver.timeout: {} {}ms", seq, ms);
        let tx = self.tx.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(ms));
            tx.send(Event::Timeout(seq)).expect("driver::timeout.send");
        });
    }

    pub fn event(&mut self) -> Result<Event> {
        self.rx.recv().chain_err(||"driverstd.event recv")
    }

    // pub fn reply (&self, _: String) {
    // }
}
