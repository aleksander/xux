use std::io::BufRead;
use byteorder::LittleEndian as LE;
use byteorder::{ReadBytesExt, WriteBytesExt};
use crate::Result;
use anyhow::anyhow;

pub trait ReadBytesSac : ReadBytesExt + BufRead {
    fn i8(&mut self) -> Result<i8> {
        Ok(self.read_i8()?)
    }
    fn i16(&mut self) -> Result<i16> {
        Ok(self.read_i16::<LE>()?)
    }
    fn i32(&mut self) -> Result<i32> {
        Ok(self.read_i32::<LE>()?)
    }
    fn i64(&mut self) -> Result<i64> {
        Ok(self.read_i64::<LE>()?)
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.read_u8()?)
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(self.read_u16::<LE>()?)
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(self.read_u32::<LE>()?)
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(self.read_u64::<LE>()?)
    }
    fn f32(&mut self) -> Result<f32> {
        Ok(self.read_f32::<LE>()?)
    }
    fn f64(&mut self) -> Result<f64> {
        Ok(self.read_f64::<LE>()?)
    }
    fn strz(&mut self) -> Result<String> {
        let mut tmp = Vec::new();
        let count = self.read_until(0, &mut tmp)?;
        if count == 0 { return Err(anyhow!("unexpected EOF")); }
        tmp.pop();
        Ok(String::from_utf8(tmp)?)
    }
    //FIXME return struct Coord
    fn coord(&mut self) -> Result<(i32,i32)> {
        Ok((self.i32()?, self.i32()?))
    }
    //FIXME return struct Color
    fn color(&mut self) -> Result<(u8,u8,u8,u8)> {
        Ok((self.u8()?, self.u8()?, self.u8()?, self.u8()?))
    }
    fn buf(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut buf = vec!(0; len);
        self.read_exact(buf.as_mut_slice())?;
        Ok(buf)
    }
}

impl<R: ReadBytesExt + BufRead + ?Sized> ReadBytesSac for R {}

pub trait WriteBytesSac : WriteBytesExt {
    fn i8(&mut self, i: i8) -> Result<()> {
        Ok(self.write_i8(i)?)
    }
    fn i16(&mut self, i: i16) -> Result<()> {
        Ok(self.write_i16::<LE>(i)?)
    }
    fn i32(&mut self, i: i32) -> Result<()> {
        Ok(self.write_i32::<LE>(i)?)
    }
    fn u8(&mut self, i: u8) -> Result<()> {
        Ok(self.write_u8(i)?)
    }
    fn u16(&mut self, i: u16) -> Result<()> {
        Ok(self.write_u16::<LE>(i)?)
    }
    fn u32(&mut self, i: u32) -> Result<()> {
        Ok(self.write_u32::<LE>(i)?)
    }
    fn f32(&mut self, i: f32) -> Result<()> {
        Ok(self.write_f32::<LE>(i)?)
    }
    fn f64(&mut self, i: f64) -> Result<()> {
        Ok(self.write_f64::<LE>(i)?)
    }
    fn strz(&mut self, i: &str) -> Result<()> {
        self.write(i.as_bytes())?;
        Ok(self.u8(0)?) //'\0'
    }
    fn coord(&mut self, x: i32, y: i32) -> Result<()> {
        self.write_i32::<LE>(x)?;
        Ok(self.write_i32::<LE>(y)?)
    }
    fn color(&mut self, r: u8, g: u8, b: u8, a: u8) -> Result<()> {
        self.write_u8(r)?;
        self.write_u8(g)?;
        self.write_u8(b)?;
        Ok(self.write_u8(a)?)
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
