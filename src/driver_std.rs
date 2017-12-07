use std;
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::io::{Read, Write};
use driver::{Driver, Event};
use Result;
use failure::err_msg;

impl Driver for DriverStd {

    // fn new (ip: std::net::IpAddr, port: u16) -> Result<&'a Driver>;

    fn tx(&self, buf: &[u8]) -> Result<()> {
        // info!("driver.tx: {} bytes", buf.len());
        let len = self.sock.send(buf)?;
        if len != buf.len() {
            return Err(err_msg("sent len != buf len"));
        }
        Ok(())
    }

    fn timeout(&self, seq: usize, ms: u64) {
        use std::time::Duration;

        // info!("driver.timeout: {} {}ms", seq, ms);
        let tx = self.tx.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(ms));
            tx.send(Event::Timeout(seq)).expect("driver::timeout.send");
        });
    }

    fn event(&mut self) -> Result<Event> {
        Ok(self.rx.recv()?)
    }
}

pub struct DriverStd {
    sock: std::net::UdpSocket,
    tx: Sender<Event>, // XXX maybe use SyncSender ?
    rx: Receiver<Event>,
}

impl DriverStd {
    pub fn new(host: &str, port: u16) -> Result<DriverStd> {
        let sock = std::net::UdpSocket::bind("0.0.0.0:0")?;
        sock.connect((host, port))?;
        let (tx,rx) = channel();

        let receiver_tx = tx.clone();
        let sock_rx = sock.try_clone()?;
        thread::spawn(move || {
            let mut buf = vec![0; 65535];
            loop {
                // TODO send Error(e) through receiver_tx
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

    pub fn sender(&self) -> Sender<Event> { self.tx.clone() }

    // pub fn reply (&self, _: String) {
    // }
}
