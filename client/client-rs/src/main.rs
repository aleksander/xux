extern crate openssl;

use std::io::Writer;
use std::io::net::tcp::TcpStream;
use std::str;

use openssl::crypto::hash::{SHA256, hash};
use openssl::ssl::{Sslv23, SslContext, SslStream/*, SslVerifyPeer*/};
//use openssl::x509::{X509Generator, X509, DigitalSignature, KeyEncipherment, ClientAuth, ServerAuth, X509StoreContext};

fn authorize(host: &str, port: u16, user: &str, pass: &str) {
    let stream = TcpStream::connect(host, port).unwrap();
    let mut stream = SslStream::new(&SslContext::new(Sslv23).unwrap(), stream).unwrap();
    stream.write_be_u16((3+user.as_bytes().len()+1+32) as u16).unwrap();
    stream.write("pw".as_bytes()).unwrap();
    stream.write_u8(0u8).unwrap();
    stream.write(user.as_bytes()).unwrap();
    stream.write_u8(0u8).unwrap();
    let pass_hash = hash(SHA256, pass.as_bytes());
    assert!(pass_hash.len() == 32u);
    stream.write(pass_hash.as_slice()).unwrap();
    stream.flush().unwrap();
//    stream.write(" there".as_bytes()).unwrap();
//    stream.flush().unwrap();
//    stream.write("GET /\r\n\r\n".as_bytes()).unwrap();
//    stream.flush().unwrap();
//    let buf = stream.read_to_end().ok().expect("read error");
//    print!("{}", str::from_utf8(buf.as_slice()));
    let length = stream.read_be_u16().ok().expect("read error");
    let msg = stream.read_exact(length as uint).ok().expect("read error");
    println!("msg='{}'", str::from_utf8(msg.as_slice()).unwrap());
}

fn main() {
    let host = "148.251.44.214";
    let port: u16 = 1871;
    let user = "salvian";
    let pass = "простойпароль";
    println!("authorize at {}:{}", host, port);
    authorize(host, port, user, pass);
}

/*
def authorize(self, name, password):
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        ss = ssl.wrap_socket(s)
        ss.connect((self.host, self.ssl_port))
        msg = bytes(bytearray([1,len(name)])+name.encode('utf8'))
        ss.write(msg)
        msg = ss.read(2)
        msg_type, length = struct.unpack('!BB', msg)
        if length > 0:
                msg = ss.read(length)
        if(msg_type != 0):
                dbg('username binding: wrong message type "'+str(msg_type)+'" '+msg)
                ss.close()
                return False
        hash = hashlib.sha256()
        hash.update(password.encode('utf8'))
        hash = hash.digest()
        msg = bytes(bytearray([2,len(hash)])+hash)
        ss.write(msg)
        msg = ss.read(2)
        msg_type, length = struct.unpack('!BB', msg)
        if length > 0:
                msg = ss.read(length)
        ss.close()
        if(msg_type != 0):
                dbg('password binding: wrong message type "'+str(msg_type)+'" '+msg)
                return False
        self.cookie = msg
        #f = open('cookie','wb')
        #f.write(self.cookie)
        #f.close()
        self.user = name
        #dbg('cookie: '+self.cookie)
        return True
*/
