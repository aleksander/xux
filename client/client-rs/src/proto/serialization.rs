use ::Error;
use ::std::io;
use ::std::io::BufRead;
use ::std::result::Result;
use ::byteorder::LittleEndian as LE;
use ::byteorder::{ReadBytesExt, WriteBytesExt};

pub trait ReadBytesSac : ReadBytesExt + BufRead {
    fn i8(&mut self) -> io::Result<i8> {
        self.read_i8()
    }
    fn i16(&mut self) -> io::Result<i16> {
        self.read_i16::<LE>()
    }
    fn i32(&mut self) -> io::Result<i32> {
        self.read_i32::<LE>()
    }
    fn u8(&mut self) -> io::Result<u8> {
        self.read_u8()
    }
    fn u16(&mut self) -> io::Result<u16> {
        self.read_u16::<LE>()
    }
    fn u32(&mut self) -> io::Result<u32> {
        self.read_u32::<LE>()
    }
    fn f32(&mut self) -> io::Result<f32> {
        self.read_f32::<LE>()
    }
    fn f64(&mut self) -> io::Result<f64> {
        self.read_f64::<LE>()
    }
    fn strz(&mut self) -> Result<String,Error> {
        let mut tmp = Vec::new();
        self.read_until(0, &mut tmp)?;
        tmp.pop();
        Ok(String::from_utf8(tmp)?)
    }
    fn coord(&mut self) -> io::Result<(i32,i32)> {
        Ok((self.i32()?, self.i32()?))
    }
    fn color(&mut self) -> io::Result<(u8,u8,u8,u8)> {
        Ok((self.u8()?, self.u8()?, self.u8()?, self.u8()?))
    }
}

impl<R: ReadBytesExt + BufRead + ?Sized> ReadBytesSac for R {}

pub trait WriteBytesSac : WriteBytesExt {
    fn i8(&mut self, i: i8) -> io::Result<()> {
        self.write_i8(i)
    }
    fn i16(&mut self, i: i16) -> io::Result<()> {
        self.write_i16::<LE>(i)
    }
    fn i32(&mut self, i: i32) -> io::Result<()> {
        self.write_i32::<LE>(i)
    }
    fn u8(&mut self, i: u8) -> io::Result<()> {
        self.write_u8(i)
    }
    fn u16(&mut self, i: u16) -> io::Result<()> {
        self.write_u16::<LE>(i)
    }
    fn u32(&mut self, i: u32) -> io::Result<()> {
        self.write_u32::<LE>(i)
    }
    fn f32(&mut self, i: f32) -> io::Result<()> {
        self.write_f32::<LE>(i)
    }
    fn f64(&mut self, i: f64) -> io::Result<()> {
        self.write_f64::<LE>(i)
    }
    fn strz(&mut self, i: &str) -> io::Result<()> {
        self.write(i.as_bytes())?;
        self.u8(0) //'\0'
    }
    fn coord(&mut self, x: i32, y: i32) -> io::Result<()> {
        self.write_i32::<LE>(x)?;
        self.write_i32::<LE>(y)
    }
    fn color(&mut self, r: u8, g: u8, b: u8, a: u8) -> io::Result<()> {
        self.write_u8(r)?;
        self.write_u8(g)?;
        self.write_u8(b)?;
        self.write_u8(a)
    }
}

impl<R: WriteBytesExt + ?Sized> WriteBytesSac for R {}

pub trait FromBuf {
    fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Vec<Self>, Error> where Self: ::std::marker::Sized;
}

pub trait ToBuf {
    fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<(), Error>;
}
