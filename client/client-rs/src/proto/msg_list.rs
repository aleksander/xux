use proto::Error;
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
    fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Vec<Self>, Error> where Self: ::std::marker::Sized {
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
                    list[deep].push(MsgList::tINT(r.i32()?));
                }
                T_STR => {
                    let tmp = r.strz()?;
                    list[deep].push(MsgList::tSTR(tmp));
                }
                T_COORD => {
                    list[deep].push(MsgList::tCOORD((r.i32()?, r.i32()?)));
                }
                T_UINT8 => {
                    list[deep].push(MsgList::tUINT8(r.u8()?));
                }
                T_UINT16 => {
                    list[deep].push(MsgList::tUINT16(r.u16()?));
                }
                T_COLOR => {
                    list[deep].push(MsgList::tCOLOR((r.u8()?, r.u8()?, r.u8()?, r.u8()?)));
                }
                T_INT8 => {
                    list[deep].push(MsgList::tINT8(r.i8()?));
                }
                T_INT16 => {
                    list[deep].push(MsgList::tINT16(r.i16()?));
                }
                T_NIL => {
                    list[deep].push(MsgList::tNIL);
                }
                T_BYTES => {
                    let len = r.u8()?;
                    if (len & 128) != 0 {
                        let len = r.i32()?;
                        assert!(len > 0);
                        let mut bytes = vec![0; len as usize];
                        r.read_exact(&mut bytes)?;
                        list[deep].push(MsgList::tBYTES(bytes));
                    } else {
                        let mut bytes = vec![0; len as usize];
                        r.read_exact(&mut bytes)?;
                        list[deep].push(MsgList::tBYTES(bytes));
                    }
                }
                T_FLOAT32 => {
                    list[deep].push(MsgList::tFLOAT32(r.f32()?));
                }
                T_FLOAT64 => {
                    list[deep].push(MsgList::tFLOAT64(r.f64()?));
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
    fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<(), Error> {
        for l in self.iter() {
            l.to_buf(w)?;
        }
        w.write_u8(T_END)?;
        Ok(())
    }
}

impl ToBuf for MsgList {
    fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<(), Error> {
        match *self {
            MsgList::tINT(i) => {
                w.u8(T_INT)?;
                w.i32(i)?;
            }
            MsgList::tSTR(ref s) => {
                w.u8(T_STR)?;
                w.strz(s)?;
            }
            MsgList::tCOORD((x, y)) => {
                w.u8(T_COORD)?;
                w.coord(x, y)?;
            }
            MsgList::tUINT8(u) => {
                w.u8(T_UINT8)?;
                w.u8(u)?;
            }
            MsgList::tUINT16(u) => {
                w.u8(T_UINT16)?;
                w.u16(u)?;
            }
            MsgList::tCOLOR((r, g, b, a)) => {
                w.u8(T_COLOR)?;
                w.color(r, g, b, a)?;
            }
            MsgList::tTTOL(_) => {
                return Err(Error {
                    source: "write_list is NOT implemented for tTTOL",
                    detail: None,
                });
            }
            MsgList::tINT8(i) => {
                w.u8(T_INT8)?;
                w.i8(i)?;
            }
            MsgList::tINT16(i) => {
                w.u8(T_INT16)?;
                w.i16(i)?;
            }
            MsgList::tNIL => {
                w.u8(T_NIL)?;
            }
            MsgList::tBYTES(_) => {
                return Err(Error {
                    source: "write_list is NOT implemented for tBYTES",
                    detail: None,
                });
            }
            MsgList::tFLOAT32(f) => {
                w.u8(T_FLOAT32)?;
                w.f32(f)?;
            }
            MsgList::tFLOAT64(f) => {
                w.u8(T_FLOAT64)?;
                w.f64(f)?;
            }
        }
        Ok(())
    }
}

/*
//TODO MsgList.from_buf()
#[deprecated]
pub fn read_list(r: &mut Cursor<&[u8]>) -> Vec<MsgList> /*TODO return Result instead*/ {
    let mut deep = 0;
    let mut list: Vec<Vec<MsgList>> = Vec::new();
    list.push(Vec::new());
    loop {
        let t = match r.read_u8() {
            Ok(b) => b,
            Err(_) => {
                while deep > 0 {
                    let tmp = list.remove(deep);
                    deep -= 1;
                    list[deep].push(MsgList::tTTOL(tmp));
                }
                return list.remove(0);
            }
        };
        match t {
            T_END => {
                if deep > 0 {
                    let tmp = list.remove(deep);
                    deep -= 1;
                    list[deep].push(MsgList::tTTOL(tmp));
                } else {
                    return list.remove(0);
                }
            }
            T_TTOL => {
                list.push(Vec::new());
                deep += 1;
            }
            T_INT => {
                list[deep].push(MsgList::tINT(r.i32().unwrap()));
            }
            T_STR => {
                let tmp = r.strz().unwrap();
                list[deep].push(MsgList::tSTR(tmp));
            }
            T_COORD => {
                list[deep].push(MsgList::tCOORD((r.i32().unwrap(), r.i32().unwrap())));
            }
            T_UINT8 => {
                list[deep].push(MsgList::tUINT8(r.read_u8().unwrap()));
            }
            T_UINT16 => {
                list[deep].push(MsgList::tUINT16(r.u16().unwrap()));
            }
            T_COLOR => {
                list[deep].push(MsgList::tCOLOR((r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap())));
            }
            T_INT8 => {
                list[deep].push(MsgList::tINT8(r.read_i8().unwrap()));
            }
            T_INT16 => {
                list[deep].push(MsgList::tINT16(r.i16().unwrap()));
            }
            T_NIL => {
                list[deep].push(MsgList::tNIL);
            }
            T_BYTES => {
                let len = r.read_u8().unwrap();
                if (len & 128) != 0 {
                    let len = r.i32().unwrap();
                    assert!(len > 0);
                    let mut bytes = vec![0; len as usize];
                    r.read_exact(&mut bytes).unwrap();
                    list[deep].push(MsgList::tBYTES(bytes));
                } else {
                    let mut bytes = vec![0; len as usize];
                    r.read_exact(&mut bytes).unwrap();
                    list[deep].push(MsgList::tBYTES(bytes));
                }
            }
            T_FLOAT32 => {
                list[deep].push(MsgList::tFLOAT32(r.f32().unwrap()));
            }
            T_FLOAT64 => {
                list[deep].push(MsgList::tFLOAT64(r.f64().unwrap()));
            }
            _ => {
                info!("    !!! UNKNOWN LIST ELEMENT !!!");
                return list.remove(0); /*TODO return Error instead*/
            }
        }
    }
}
*/
