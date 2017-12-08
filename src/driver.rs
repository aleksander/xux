use std;
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use Result;
use failure::err_msg;

#[derive(Debug)]
pub enum Event {
    Rx(Vec<u8>),
    Timeout(usize),
    Render(RenderEvent),
}

#[derive(Debug)]
pub enum RenderEvent {
    Up,
    Down,
    Left,
    Right,
    Quit,
}

pub struct Driver {
    sock: std::net::UdpSocket,
    tx: Sender<Event>, // XXX maybe use SyncSender ?
    rx: Receiver<Event>,
}

pub fn new(host: &str, port: u16) -> Result<Driver> {
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

    Ok(Driver {
        sock: sock,
        tx: tx,
        rx: rx,
    })
}

impl Driver {
    pub fn sender(&self) -> Sender<Event> {
        self.tx.clone()
    }

    pub fn transmit(&self, buf: &[u8]) -> Result<()> {
        // info!("driver.tx: {} bytes", buf.len());
        let len = self.sock.send(buf)?;
        if len != buf.len() {
            return Err(err_msg("sent len != buf len"));
        }
        Ok(())
    }

    pub fn add_timeout(&self, seq: usize, ms: u64) {
        use std::time::Duration;

        // info!("driver.timeout: {} {}ms", seq, ms);
        let tx = self.tx.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(ms));
            tx.send(Event::Timeout(seq)).expect("driver::timeout.send");
        });
    }

    pub fn next_event(&mut self) -> Result<Event> {
        Ok(self.rx.recv()?)
    }

    // pub fn reply (&self, _: String) {
    // }
}
