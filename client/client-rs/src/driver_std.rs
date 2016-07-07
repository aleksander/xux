use std;
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::RecvError;
use std::io::Read;
use std::io::Write;

use driver::{Driver, Event};

impl Driver for DriverStd {
    // fn new (ip: std::net::IpAddr, port: u16) -> std::io::Result<&'a Driver>;
    fn tx(&self, buf: &[u8]) -> std::io::Result<()> {
        self.tx(buf)
    }
    fn timeout(&self, seq: usize, ms: u64) {
        self.timeout(seq, ms);
    }
    fn event(&mut self) -> Result<Event, RecvError> {
        self.event()
    }
}

pub struct DriverStd {
    sock: std::net::UdpSocket,
    // buf: Vec<u8>,
    dst: std::net::SocketAddr,
    tx: Sender<Event>, // XXX maybe use SyncSender ?
    rx: Receiver<Event>,
}

impl DriverStd {
    pub fn new(ip: std::net::IpAddr, port: u16) -> std::io::Result<DriverStd> {
        let sock = std::net::UdpSocket::bind("0.0.0.0:0")?;
        let (tx, rx) = channel();
        let dst = std::net::SocketAddr::new(ip, port);

        let _tx = tx.clone();
        let _sock = sock.try_clone().expect("driver::new.try_clone(sock)");
        let serv = dst;
        thread::spawn(move || {
            let mut buf = vec![0; 65535];
            loop {
                // FIXME check the sender ip:port
                let (len, src) = _sock.recv_from(&mut buf).expect("driver::recv_from(sock)");
                if src != serv {
                    info!("WARNING: datagram not from serv");
                    continue;
                }
                // TODO zero-copy data processing
                _tx.send(Event::Rx(buf[..len].to_vec())).expect("driver::send(event::rx)");
            }
        });

        let _tx = tx.clone();
        thread::spawn(move || {
            let listener = std::net::TcpListener::bind("127.0.0.1:8080").expect("driver::new.tcp_new");
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let _tx = _tx.clone();
                        thread::spawn(move || {
                            let mut buf = vec![0; 1024];
                            let (reply_tx, reply_rx) = channel();
                            loop {
                                let len = stream.read(&mut buf).expect("driver::tcp_stream.read");
                                _tx.send(Event::Tcp((reply_tx.clone(), buf[..len].to_vec())))
                                    .expect("driver::send(event::tcp)");
                                let reply = reply_rx.recv().expect("driver::recv_rx");
                                // info!("RENDERRED REPLY: {:?}", reply);
                                let /*len*/_ = stream.write(reply.as_bytes()).expect("strem.write");
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
            // buf: vec![0; 65535],
            dst: dst,
            tx: tx,
            rx: rx,
        })
    }

    pub fn tx(&self, buf: &[u8]) -> std::io::Result<()> {
        // info!("driver.tx: {} bytes", buf.len());
        let len = self.sock.send_to(buf, &self.dst)?;
        if len != buf.len() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "sent len != buf len"));
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

    pub fn event(&mut self) -> Result<Event, std::sync::mpsc::RecvError> {
        self.rx.recv()
    }

    // pub fn reply (&self, _: String) {
    // }
}
