use std;
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
//use std::sync::mpsc;
use std::io::Read;
use std::io::Write;

pub struct Driver {
    sock: std::net::UdpSocket,
    //buf: Vec<u8>,
    dst: std::net::SocketAddr,
    tx: Sender<Event>, //XXX maybe use SyncSender ?
    rx: Receiver<Event>,
}

impl Driver {
    pub fn new (ip: std::net::IpAddr, port: u16) -> std::io::Result<Driver> {
        let sock = try!(std::net::UdpSocket::bind("0.0.0.0:0"));
        let (tx, rx) = channel();
        let dst = std::net::SocketAddr::new(ip, port);

        let _tx = tx.clone();
        let _sock = sock.try_clone().unwrap();
        let serv = dst;
        thread::spawn(move || {
            let mut buf = vec![0; 65535];
            loop {
                //FIXME check the sender ip:port
                let (len, src) = _sock.recv_from(&mut buf).unwrap();
                if src != serv {
                    println!("WARNING: datagram not from serv");
                    continue;
                }
                //TODO zero-copy data processing
                _tx.send(Event::Rx(buf[..len].to_vec())).unwrap();
            }
        });

        let _tx = tx.clone();
        thread::spawn(move || {
            let listener = std::net::TcpListener::bind("127.0.0.1:8080").unwrap();
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let _tx = _tx.clone();
                        thread::spawn(move || {
                            let mut buf = vec![0; 1024];
                            let (reply_tx, reply_rx) = channel();
                            loop {
                                let len = stream.read(&mut buf).unwrap();
                                _tx.send(Event::Tcp( (reply_tx.clone(), buf[..len].to_vec()) )).unwrap();
                                let reply = reply_rx.recv().unwrap();
                                //println!("RENDERRED REPLY: {:?}", reply);
                                let /*len*/_ = stream.write(reply.as_bytes()).unwrap();
                            }
                        });
                    }
                    Err(e) => { println!("connection error: {:?}", e); break; }
                }
            }
        });

        Ok(Driver{
            sock: sock,
            //buf: vec![0; 65535],
            dst: dst,
            tx: tx,
            rx: rx
        })
    }

    pub fn tx (&self, buf: &[u8]) -> std::io::Result<()> {
        //println!("driver.tx: {} bytes", buf.len());
        let len = try!(self.sock.send_to(buf, &self.dst));
        if len != buf.len() { return Err(std::io::Error::new(std::io::ErrorKind::Other, "sent len != buf len")) }
        Ok(())
    }

    pub fn timeout (&self, seq: usize, ms: u64) {
        //println!("driver.timeout: {} {}ms", seq, ms);
        let tx = self.tx.clone();
        thread::spawn(move || {
            thread::sleep_ms(ms as u32);
            tx.send(Event::Timeout(seq)).unwrap();
        });
    }

    pub fn event (&mut self) -> std::result::Result<Event, std::sync::mpsc::RecvError> {
        self.rx.recv()
    }

    //pub fn reply (&self, _: String) {
    //}
}

//TODO move to driver trait module
pub enum Event {
    Rx(Vec<u8>),
    Timeout(usize),
    Tcp( (Sender<String>,Vec<u8>) ),
}
