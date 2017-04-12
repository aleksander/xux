use ::errors::*;
use proto::serialization::*;

#[allow(non_camel_case_types)]
#[derive(Debug)]
// TODO replace with plain struct variants
pub enum MsgList {
    tINT(i32),
    tSTR(String),
    tCOORD((i32, i32)),
    tUINT8(u8),
    tUINT16(u16),
    tCOLOR((u8, u8, u8, u8)),
    tTTOL(Vec<MsgList>),
    tINT8(i8),
    tINT16(i16),
    tNIL,
    tBYTES(Vec<u8>),
    tFLOAT32(f32),
    tFLOAT64(f64),
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
                        list[deep].push(MsgList::tTTOL(tmp));
                    }
                    return Ok(list.remove(0));
                }
            };
            match t {
                T_END => {
                    if deep > 0 {
                        let tmp = list.remove(deep);
                        deep -= 1;
                        list[deep].push(MsgList::tTTOL(tmp));
                    } else {
                        return Ok(list.remove(0));
                    }
                }
                T_TTOL => {
                    list.push(Vec::new());
                    deep += 1;
                }
                T_INT => {
                    list[deep].push(MsgList::tINT(r.i32().chain_err(||"list INT")?));
                }
                T_STR => {
                    list[deep].push(MsgList::tSTR(r.strz()?));
                }
                T_COORD => {
                    list[deep].push(MsgList::tCOORD(r.coord().chain_err(||"list COORD")?));
                }
                T_UINT8 => {
                    list[deep].push(MsgList::tUINT8(r.u8().chain_err(||"list UINT8")?));
                }
                T_UINT16 => {
                    list[deep].push(MsgList::tUINT16(r.u16().chain_err(||"list UINT16")?));
                }
                T_COLOR => {
                    list[deep].push(MsgList::tCOLOR(r.color().chain_err(||"list COLOR")?));
                }
                T_INT8 => {
                    list[deep].push(MsgList::tINT8(r.i8().chain_err(||"list INT8")?));
                }
                T_INT16 => {
                    list[deep].push(MsgList::tINT16(r.i16().chain_err(||"list INT16")?));
                }
                T_NIL => {
                    list[deep].push(MsgList::tNIL);
                }
                T_BYTES => {
                    let len = r.u8().chain_err(||"list BYTES len")?;
                    if (len & 128) != 0 {
                        let len = r.i32().chain_err(||"list BYTES len2")?;
                        if len <= 0 { return Err("MsgList.from_buf: len <= 0".into()); }
                        //TODO this magic 65535 const should be set to some adequate default (but what is adequate?)
                        if len > 65535 { return Err("MsgList.from_buf: len > 65535".into()); }
                        let mut bytes = vec![0; len as usize];
                        r.read_exact(&mut bytes).chain_err(||"list read bytes")?;
                        list[deep].push(MsgList::tBYTES(bytes));
                    } else {
                        let mut bytes = vec![0; len as usize];
                        r.read_exact(&mut bytes).chain_err(||"list read bytes2")?;
                        list[deep].push(MsgList::tBYTES(bytes));
                    }
                }
                T_FLOAT32 => {
                    list[deep].push(MsgList::tFLOAT32(r.f32().chain_err(||"list FLOAT32")?));
                }
                T_FLOAT64 => {
                    list[deep].push(MsgList::tFLOAT64(r.f64().chain_err(||"list FLOAT64")?));
                }
                _ => {
                    info!("    !!! UNKNOWN LIST ELEMENT !!!");
                    return Ok(list.remove(0));
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
            MsgList::tINT(i) => {
                w.u8(T_INT).chain_err(||"list to_buf INT")?;
                w.i32(i).chain_err(||"list to_buf INT value")?;
            }
            MsgList::tSTR(ref s) => {
                w.u8(T_STR).chain_err(||"list to_buf STR")?;
                w.strz(s).chain_err(||"list to_buf STR value")?;
            }
            MsgList::tCOORD((x, y)) => {
                w.u8(T_COORD).chain_err(||"list to_buf COORD")?;
                w.coord(x, y).chain_err(||"list to_buf COORD value")?;
            }
            MsgList::tUINT8(u) => {
                w.u8(T_UINT8).chain_err(||"list to_buf UINT8")?;
                w.u8(u).chain_err(||"list to_buf UINT8 value")?;
            }
            MsgList::tUINT16(u) => {
                w.u8(T_UINT16).chain_err(||"list to_buf UINT16")?;
                w.u16(u).chain_err(||"list to_buf UINT16 value")?;
            }
            MsgList::tCOLOR((r, g, b, a)) => {
                w.u8(T_COLOR).chain_err(||"list to_buf COLOR")?;
                w.color(r, g, b, a).chain_err(||"list to_buf COLOR value")?;
            }
            MsgList::tTTOL(_) => {
                return Err("list.to_buf is NOT implemented for TTOL".into());
            }
            MsgList::tINT8(i) => {
                w.u8(T_INT8).chain_err(||"list to_buf INT8")?;
                w.i8(i).chain_err(||"list to_buf INT8 value")?;
            }
            MsgList::tINT16(i) => {
                w.u8(T_INT16).chain_err(||"list to_buf INT16")?;
                w.i16(i).chain_err(||"list to_buf INT16 value")?;
            }
            MsgList::tNIL => {
                w.u8(T_NIL).chain_err(||"list to_buf NIL")?;
            }
            MsgList::tBYTES(_) => {
                return Err("list.to_buf is NOT implemented for BYTES".into());
            }
            MsgList::tFLOAT32(f) => {
                w.u8(T_FLOAT32).chain_err(||"list to_buf FLOAT32")?;
                w.f32(f).chain_err(||"list to_buf FLOAT32 value")?;
            }
            MsgList::tFLOAT64(f) => {
                w.u8(T_FLOAT64).chain_err(||"list to_buf FLOAT64")?;
                w.f64(f).chain_err(||"list to_buf FLOAT64 value")?;
            }
        }
        Ok(())
    }
}
