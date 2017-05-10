use ::errors::*;
use proto::serialization::*;
use proto::Color;

#[derive(Debug)]
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
                    list[deep].push(List::Int(r.i32().chain_err(||"list INT")?));
                }
                T_STR => {
                    list[deep].push(List::Str(r.strz()?));
                }
                T_COORD => {
                    list[deep].push(List::Coord(r.coord().chain_err(||"list COORD")?));
                }
                T_UINT8 => {
                    list[deep].push(List::Uint8(r.u8().chain_err(||"list UINT8")?));
                }
                T_UINT16 => {
                    list[deep].push(List::Uint16(r.u16().chain_err(||"list UINT16")?));
                }
                T_COLOR => {
                    list[deep].push(List::Color(r.color().chain_err(||"list COLOR")?));
                }
                T_INT8 => {
                    list[deep].push(List::Int8(r.i8().chain_err(||"list INT8")?));
                }
                T_INT16 => {
                    list[deep].push(List::Int16(r.i16().chain_err(||"list INT16")?));
                }
                T_NIL => {
                    list[deep].push(List::Nil);
                }
                T_BYTES => {
                    let len = r.u8().chain_err(||"list BYTES len")?;
                    if (len & 128) != 0 {
                        let len = r.i32().chain_err(||"list BYTES len2")?;
                        if len <= 0 { return Err("List.from_buf: len <= 0".into()); }
                        //TODO this magic 65535 const should be set to some adequate default (but what is adequate?)
                        if len > 1024*1024 { return Err("List.from_buf: len > MiB".into()); }
                        let mut bytes = vec![0; len as usize];
                        r.read_exact(&mut bytes).chain_err(||"list read bytes")?;
                        list[deep].push(List::Bytes(bytes));
                    } else {
                        let mut bytes = vec![0; len as usize];
                        r.read_exact(&mut bytes).chain_err(||"list read bytes2")?;
                        list[deep].push(List::Bytes(bytes));
                    }
                }
                T_FLOAT32 => {
                    list[deep].push(List::Float32(r.f32().chain_err(||"list FLOAT32")?));
                }
                T_FLOAT64 => {
                    list[deep].push(List::Float64(r.f64().chain_err(||"list FLOAT64")?));
                }
                _ => {
                    return Err(format!("unknown MstList type: {}", t).into());
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
        w.u8(T_END).chain_err(||"list to_buf END")?;
        Ok(())
    }
}

impl ToBuf for List {
    fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<()> {
        match *self {
            List::Int(i) => {
                w.u8(T_INT).chain_err(||"list to_buf INT")?;
                w.i32(i).chain_err(||"list to_buf INT value")?;
            }
            List::Str(ref s) => {
                w.u8(T_STR).chain_err(||"list to_buf STR")?;
                w.strz(s).chain_err(||"list to_buf STR value")?;
            }
            List::Coord((x, y)) => {
                w.u8(T_COORD).chain_err(||"list to_buf COORD")?;
                w.coord(x, y).chain_err(||"list to_buf COORD value")?;
            }
            List::Uint8(u) => {
                w.u8(T_UINT8).chain_err(||"list to_buf UINT8")?;
                w.u8(u).chain_err(||"list to_buf UINT8 value")?;
            }
            List::Uint16(u) => {
                w.u8(T_UINT16).chain_err(||"list to_buf UINT16")?;
                w.u16(u).chain_err(||"list to_buf UINT16 value")?;
            }
            List::Color((r, g, b, a)) => {
                w.u8(T_COLOR).chain_err(||"list to_buf COLOR")?;
                w.color(r, g, b, a).chain_err(||"list to_buf COLOR value")?;
            }
            List::Ttol(_) => {
                return Err("list.to_buf is NOT implemented for TTOL".into());
            }
            List::Int8(i) => {
                w.u8(T_INT8).chain_err(||"list to_buf INT8")?;
                w.i8(i).chain_err(||"list to_buf INT8 value")?;
            }
            List::Int16(i) => {
                w.u8(T_INT16).chain_err(||"list to_buf INT16")?;
                w.i16(i).chain_err(||"list to_buf INT16 value")?;
            }
            List::Nil => {
                w.u8(T_NIL).chain_err(||"list to_buf NIL")?;
            }
            List::Bytes(_) => {
                return Err("list.to_buf is NOT implemented for BYTES".into());
            }
            List::Float32(f) => {
                w.u8(T_FLOAT32).chain_err(||"list to_buf FLOAT32")?;
                w.f32(f).chain_err(||"list to_buf FLOAT32 value")?;
            }
            List::Float64(f) => {
                w.u8(T_FLOAT64).chain_err(||"list to_buf FLOAT64")?;
                w.f64(f).chain_err(||"list to_buf FLOAT64 value")?;
            }
        }
        Ok(())
    }
}
