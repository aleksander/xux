use errors::*;
use std::io::BufRead;
use byteorder::LittleEndian as LE;
use byteorder::{ReadBytesExt, WriteBytesExt};

pub trait ReadBytesSac : ReadBytesExt + BufRead {
    fn i8(&mut self) -> Result<i8> {
        self.read_i8().chain_err(||"read_i8")
    }
    fn i16(&mut self) -> Result<i16> {
        self.read_i16::<LE>().chain_err(||"read_i16")
    }
    fn i32(&mut self) -> Result<i32> {
        self.read_i32::<LE>().chain_err(||"read_i32")
    }
    fn i64(&mut self) -> Result<i64> {
        self.read_i64::<LE>().chain_err(||"read_i64")
    }
    fn u8(&mut self) -> Result<u8> {
        self.read_u8().chain_err(||"read_u8")
    }
    fn u16(&mut self) -> Result<u16> {
        self.read_u16::<LE>().chain_err(||"read_u16")
    }
    fn u32(&mut self) -> Result<u32> {
        self.read_u32::<LE>().chain_err(||"read_u32")
    }
    fn u64(&mut self) -> Result<u64> {
        self.read_u64::<LE>().chain_err(||"read_u64")
    }
    fn f32(&mut self) -> Result<f32> {
        self.read_f32::<LE>().chain_err(||"read_f32")
    }
    fn f64(&mut self) -> Result<f64> {
        self.read_f64::<LE>().chain_err(||"read_f64")
    }
    fn strz(&mut self) -> Result<String> {
        let mut tmp = Vec::new();
        let count = self.read_until(0, &mut tmp).chain_err(||"strz read_until")?;
        if count == 0 { return Err("unexpected EOF".into()); }
        tmp.pop();
        Ok(String::from_utf8(tmp).chain_err(||"strz from_utf8")?)
    }
    //FIXME return struct Coord
    fn coord(&mut self) -> Result<(i32,i32)> {
        Ok((self.i32()?, self.i32()?))
    }
    //FIXME return struct Color
    fn color(&mut self) -> Result<(u8,u8,u8,u8)> {
        Ok((self.u8()?, self.u8()?, self.u8()?, self.u8()?))
    }
}

impl<R: ReadBytesExt + BufRead + ?Sized> ReadBytesSac for R {}

pub trait WriteBytesSac : WriteBytesExt {
    fn i8(&mut self, i: i8) -> Result<()> {
        self.write_i8(i).chain_err(||"write_i8")
    }
    fn i16(&mut self, i: i16) -> Result<()> {
        self.write_i16::<LE>(i).chain_err(||"write_i16")
    }
    fn i32(&mut self, i: i32) -> Result<()> {
        self.write_i32::<LE>(i).chain_err(||"write_i32")
    }
    fn u8(&mut self, i: u8) -> Result<()> {
        self.write_u8(i).chain_err(||"write_u8")
    }
    fn u16(&mut self, i: u16) -> Result<()> {
        self.write_u16::<LE>(i).chain_err(||"write_u16")
    }
    fn u32(&mut self, i: u32) -> Result<()> {
        self.write_u32::<LE>(i).chain_err(||"write_u32")
    }
    fn f32(&mut self, i: f32) -> Result<()> {
        self.write_f32::<LE>(i).chain_err(||"write_f32")
    }
    fn f64(&mut self, i: f64) -> Result<()> {
        self.write_f64::<LE>(i).chain_err(||"write_f64")
    }
    fn strz(&mut self, i: &str) -> Result<()> {
        self.write(i.as_bytes()).chain_err(||"write_strz str")?;
        self.u8(0).chain_err(||"write_strz 0") //'\0'
    }
    fn coord(&mut self, x: i32, y: i32) -> Result<()> {
        self.write_i32::<LE>(x).chain_err(||"write_coord x")?;
        self.write_i32::<LE>(y).chain_err(||"write_coord y")
    }
    fn color(&mut self, r: u8, g: u8, b: u8, a: u8) -> Result<()> {
        self.write_u8(r).chain_err(||"write_color r")?;
        self.write_u8(g).chain_err(||"write_color g")?;
        self.write_u8(b).chain_err(||"write_color b")?;
        self.write_u8(a).chain_err(||"write_color a")
    }
}

impl<R: WriteBytesExt + ?Sized> WriteBytesSac for R {}

pub trait FromBuf {
    //FIXME should return Self, not Vec<Self>
    fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Vec<Self>> where Self: ::std::marker::Sized;
}

pub trait ToBuf {
    fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<()>;
}
