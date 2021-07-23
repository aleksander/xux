use crate::proto::serialization::*;
use crate::proto::Color;
use crate::Result;
use failure::{err_msg, format_err};
use serde::{Serialize, Deserialize};

#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum List {
    Int(i32),
    Str(String),
    Coord((i32,i32)),
    Uint8(u8),
    Uint16(u16),
    Color(Color),
    Ttol(Vec<List>),
    Int8(i8),
    Int16(i16),
    Nil,
    Bytes(Vec<u8>),
    Float32(f32),
    Float64(f64),
    FCoord32((f32,f32)),
    FCoord64((f64,f64)),
}

const T_END: u8 = 0;
const T_TTOL: u8 = 8;
const T_INT: u8 = 1;
const T_STR: u8 = 2;
const T_COORD: u8 = 3;
const T_UINT8: u8 = 4;
const T_UINT16: u8 = 5;
const T_COLOR: u8 = 6;
const T_INT8: u8 = 9;
const T_INT16: u8 = 10;
const T_NIL: u8 = 12;
const T_BYTES: u8 = 14;
const T_FLOAT32: u8 = 15;
const T_FLOAT64: u8 = 16;
const T_FCOORD32: u8 = 18;
const T_FCOORD64: u8 = 19;


impl FromBuf for List {
    fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Vec<Self>> where Self: ::std::marker::Sized {
        let mut deep = 0;
        let mut list: Vec<Vec<List>> = Vec::new();
        list.push(Vec::new());
        loop {
            let t = match r.u8() {
                Ok(b) => b,
                Err(_) => {
                    while deep > 0 {
                        let tmp = list.remove(deep);
                        deep -= 1;
                        list[deep].push(List::Ttol(tmp));
                    }
                    return Ok(list.remove(0));
                }
            };
            match t {
                T_END => {
                    if deep > 0 {
                        let tmp = list.remove(deep);
                        deep -= 1;
                        list[deep].push(List::Ttol(tmp));
                    } else {
                        return Ok(list.remove(0));
                    }
                }
                T_TTOL => {
                    list.push(Vec::new());
                    deep += 1;
                }
                T_INT => {
                    list[deep].push(List::Int(r.i32()?));
                }
                T_STR => {
                    list[deep].push(List::Str(r.strz()?));
                }
                T_COORD => {
                    list[deep].push(List::Coord(r.coord()?));
                }
                T_UINT8 => {
                    list[deep].push(List::Uint8(r.u8()?));
                }
                T_UINT16 => {
                    list[deep].push(List::Uint16(r.u16()?));
                }
                T_COLOR => {
                    list[deep].push(List::Color(r.color()?));
                }
                T_INT8 => {
                    list[deep].push(List::Int8(r.i8()?));
                }
                T_INT16 => {
                    list[deep].push(List::Int16(r.i16()?));
                }
                T_NIL => {
                    list[deep].push(List::Nil);
                }
                T_BYTES => {
                    let len = r.u8()?;
                    if (len & 128) != 0 {
                        let len = r.i32()?;
                        if len <= 0 { return Err(err_msg("List.from_buf: len <= 0")); }
                        //TODO this magic 65535 const should be set to some adequate default (but what is adequate?)
                        if len > 1024*1024 { return Err(err_msg("List.from_buf: len > MiB")); }
                        let mut bytes = vec![0; len as usize];
                        r.read_exact(&mut bytes)?;
                        list[deep].push(List::Bytes(bytes));
                    } else {
                        let mut bytes = vec![0; len as usize];
                        r.read_exact(&mut bytes)?;
                        list[deep].push(List::Bytes(bytes));
                    }
                }
                T_FLOAT32 => {
                    list[deep].push(List::Float32(r.f32()?));
                }
                T_FLOAT64 => {
                    list[deep].push(List::Float64(r.f64()?));
                }
                T_FCOORD32 => {
                    list[deep].push(List::FCoord32((r.f32()?, r.f32()?)));
                }
                T_FCOORD64 => {
                    list[deep].push(List::FCoord64((r.f64()?, r.f64()?)));
                }
                _ => {
                    return Err(format_err!("unknown MstList type: {}", t));
                }
            }
        }
    }
}

impl ToBuf for [List] {
    fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<()> {
        for l in self.iter() {
            l.to_buf(w)?;
        }
        w.u8(T_END)?;
        Ok(())
    }
}

impl ToBuf for List {
    fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<()> {
        match *self {
            List::Int(i) => {
                w.u8(T_INT)?;
                w.i32(i)?;
            }
            List::Str(ref s) => {
                w.u8(T_STR)?;
                w.strz(s)?;
            }
            List::Coord((x, y)) => {
                w.u8(T_COORD)?;
                w.coord(x, y)?;
            }
            List::Uint8(u) => {
                w.u8(T_UINT8)?;
                w.u8(u)?;
            }
            List::Uint16(u) => {
                w.u8(T_UINT16)?;
                w.u16(u)?;
            }
            List::Color((r, g, b, a)) => {
                w.u8(T_COLOR)?;
                w.color(r, g, b, a)?;
            }
            List::Ttol(_) => {
                return Err(err_msg("list.to_buf is NOT implemented for TTOL"));
            }
            List::Int8(i) => {
                w.u8(T_INT8)?;
                w.i8(i)?;
            }
            List::Int16(i) => {
                w.u8(T_INT16)?;
                w.i16(i)?;
            }
            List::Nil => {
                w.u8(T_NIL)?;
            }
            List::Bytes(_) => {
                return Err(err_msg("list.to_buf is NOT implemented for BYTES"));
            }
            List::Float32(f) => {
                w.u8(T_FLOAT32)?;
                w.f32(f)?;
            }
            List::Float64(f) => {
                w.u8(T_FLOAT64)?;
                w.f64(f)?;
            }
            List::FCoord32((x,y)) => {
                w.u8(T_FCOORD32)?;
                w.f32(x)?;
                w.f32(y)?;
            }
            List::FCoord64((x,y)) => {
                w.u8(T_FCOORD64)?;
                w.f64(x)?;
                w.f64(y)?;
            }
        }
        Ok(())
    }
}
