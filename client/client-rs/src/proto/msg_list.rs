use ::errors::*;
use proto::serialization::*;
use proto::{Coord, Color};

#[derive(Debug)]
pub enum MsgList {
    Int(i32),
    Str(String),
    Coord(Coord),
    Uint8(u8),
    Uint16(u16),
    Color(Color),
    Ttol(Vec<MsgList>),
    Int8(i8),
    Int16(i16),
    Nil,
    Bytes(Vec<u8>),
    Float32(f32),
    Float64(f64),
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

impl FromBuf for MsgList {
    fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Vec<Self>> where Self: ::std::marker::Sized {
        let mut deep = 0;
        let mut list: Vec<Vec<MsgList>> = Vec::new();
        list.push(Vec::new());
        loop {
            let t = match r.u8() {
                Ok(b) => b,
                Err(_) => {
                    while deep > 0 {
                        let tmp = list.remove(deep);
                        deep -= 1;
                        list[deep].push(MsgList::Ttol(tmp));
                    }
                    return Ok(list.remove(0));
                }
            };
            match t {
                T_END => {
                    if deep > 0 {
                        let tmp = list.remove(deep);
                        deep -= 1;
                        list[deep].push(MsgList::Ttol(tmp));
                    } else {
                        return Ok(list.remove(0));
                    }
                }
                T_TTOL => {
                    list.push(Vec::new());
                    deep += 1;
                }
                T_INT => {
                    list[deep].push(MsgList::Int(r.i32().chain_err(||"list INT")?));
                }
                T_STR => {
                    list[deep].push(MsgList::Str(r.strz()?));
                }
                T_COORD => {
                    list[deep].push(MsgList::Coord(r.coord().chain_err(||"list COORD")?));
                }
                T_UINT8 => {
                    list[deep].push(MsgList::Uint8(r.u8().chain_err(||"list UINT8")?));
                }
                T_UINT16 => {
                    list[deep].push(MsgList::Uint16(r.u16().chain_err(||"list UINT16")?));
                }
                T_COLOR => {
                    list[deep].push(MsgList::Color(r.color().chain_err(||"list COLOR")?));
                }
                T_INT8 => {
                    list[deep].push(MsgList::Int8(r.i8().chain_err(||"list INT8")?));
                }
                T_INT16 => {
                    list[deep].push(MsgList::Int16(r.i16().chain_err(||"list INT16")?));
                }
                T_NIL => {
                    list[deep].push(MsgList::Nil);
                }
                T_BYTES => {
                    let len = r.u8().chain_err(||"list BYTES len")?;
                    if (len & 128) != 0 {
                        let len = r.i32().chain_err(||"list BYTES len2")?;
                        if len <= 0 { return Err("MsgList.from_buf: len <= 0".into()); }
                        //TODO this magic 65535 const should be set to some adequate default (but what is adequate?)
                        if len > 1024*1024 { return Err("MsgList.from_buf: len > MiB".into()); }
                        let mut bytes = vec![0; len as usize];
                        r.read_exact(&mut bytes).chain_err(||"list read bytes")?;
                        list[deep].push(MsgList::Bytes(bytes));
                    } else {
                        let mut bytes = vec![0; len as usize];
                        r.read_exact(&mut bytes).chain_err(||"list read bytes2")?;
                        list[deep].push(MsgList::Bytes(bytes));
                    }
                }
                T_FLOAT32 => {
                    list[deep].push(MsgList::Float32(r.f32().chain_err(||"list FLOAT32")?));
                }
                T_FLOAT64 => {
                    list[deep].push(MsgList::Float64(r.f64().chain_err(||"list FLOAT64")?));
                }
                _ => {
                    return Err(format!("unknown MstList type: {}", t).into());
                }
            }
        }
    }
}

impl ToBuf for [MsgList] {
    fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<()> {
        for l in self.iter() {
            l.to_buf(w)?;
        }
        w.u8(T_END).chain_err(||"list to_buf END")?;
        Ok(())
    }
}

impl ToBuf for MsgList {
    fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<()> {
        match *self {
            MsgList::Int(i) => {
                w.u8(T_INT).chain_err(||"list to_buf INT")?;
                w.i32(i).chain_err(||"list to_buf INT value")?;
            }
            MsgList::Str(ref s) => {
                w.u8(T_STR).chain_err(||"list to_buf STR")?;
                w.strz(s).chain_err(||"list to_buf STR value")?;
            }
            MsgList::Coord((x, y)) => {
                w.u8(T_COORD).chain_err(||"list to_buf COORD")?;
                w.coord(x, y).chain_err(||"list to_buf COORD value")?;
            }
            MsgList::Uint8(u) => {
                w.u8(T_UINT8).chain_err(||"list to_buf UINT8")?;
                w.u8(u).chain_err(||"list to_buf UINT8 value")?;
            }
            MsgList::Uint16(u) => {
                w.u8(T_UINT16).chain_err(||"list to_buf UINT16")?;
                w.u16(u).chain_err(||"list to_buf UINT16 value")?;
            }
            MsgList::Color((r, g, b, a)) => {
                w.u8(T_COLOR).chain_err(||"list to_buf COLOR")?;
                w.color(r, g, b, a).chain_err(||"list to_buf COLOR value")?;
            }
            MsgList::Ttol(_) => {
                return Err("list.to_buf is NOT implemented for TTOL".into());
            }
            MsgList::Int8(i) => {
                w.u8(T_INT8).chain_err(||"list to_buf INT8")?;
                w.i8(i).chain_err(||"list to_buf INT8 value")?;
            }
            MsgList::Int16(i) => {
                w.u8(T_INT16).chain_err(||"list to_buf INT16")?;
                w.i16(i).chain_err(||"list to_buf INT16 value")?;
            }
            MsgList::Nil => {
                w.u8(T_NIL).chain_err(||"list to_buf NIL")?;
            }
            MsgList::Bytes(_) => {
                return Err("list.to_buf is NOT implemented for BYTES".into());
            }
            MsgList::Float32(f) => {
                w.u8(T_FLOAT32).chain_err(||"list to_buf FLOAT32")?;
                w.f32(f).chain_err(||"list to_buf FLOAT32 value")?;
            }
            MsgList::Float64(f) => {
                w.u8(T_FLOAT64).chain_err(||"list to_buf FLOAT64")?;
                w.f64(f).chain_err(||"list to_buf FLOAT64 value")?;
            }
        }
        Ok(())
    }
}
