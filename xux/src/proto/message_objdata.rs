use std::fmt;
use std::fmt::Formatter;
use crate::proto::serialization::*;
use crate::proto::ObjXY;
use crate::Result;
use std::f64::consts::PI;
use anyhow::anyhow;

pub struct ObjData {
    pub obj: Vec<ObjDataElem>,
}

impl ObjData {
    pub const ID: u8 = 6;

    pub fn new (obj: Vec<ObjDataElem>) -> ObjData {
        ObjData {
            obj: obj
        }
    }

    // TODO impl FromBuf for ObjData {}
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<ObjData> {
        let mut obj = Vec::new();
        //TODO let obj = ObjDataElem::iter(r).collect::<Result<Vec<ObjDataElem>>>().chain_err(||"")?;
        while let Some(o) = ObjDataElem::from_buf(r)? {
            obj.push(o);
        }
        Ok(ObjData { obj: obj })
    }
}

impl fmt::Debug for ObjData {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "OBJDATA")?;
        for o in &self.obj {
            writeln!(f, "      {:?}", o)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ObjDataElem {
    pub fl: u8,
    pub id: u32,
    pub frame: i32,
    pub prop: Vec<ObjDataElemProp>,
}

impl ObjDataElem {
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Option<ObjDataElem>> {
        let fl = match r.u8() {
            Ok(b) => b,
            Err(_) => { //TODO check error type
                return Ok(None);
            }
        };
        let id = r.u32()?;
        let frame = r.i32()?;
        let mut prop = Vec::new();
        //TODO let props = ObjDataElemProp::iter(r).collect::<Result<Vec<ObjDataElemProp>>>().chain_err(||"")?;
        while let Some(p) = ObjDataElemProp::from_buf(r)? {
            prop.push(p);
        }
        Ok(Some(ObjDataElem{
            fl: fl,
            id: id,
            frame: frame,
            prop: prop,
        }))
    }
}

#[derive(Debug)]
pub enum ObjDataElemProp {
    Rem,
    //TODO Move(ObjXY, Rotation)
    Move(ObjXY, f64),
    Res(u16),
    Linbeg(Linbeg),
    Linstep(Linstep),
    Speech(u16, String),
    Compose(u16),
    Zoff(Zoff),
    Lumin((i32, i32), u16, u8),
    Avatar(Vec<u16>),
    Follow(Follow),
    Homing(Homing),
    Overlay(u16),
    Auth,
    Health(u8),
    Buddy(Buddy),
    Cmppose(u8, Option<Vec<u16>>, Option<(Option<Vec<u16>>, u8)>),
    Cmpmod(Option<Vec<Vec<u16>>>),
    Cmpequ(Option<Vec<(u8, String, u16, Option<(u16, u16, u16)>)>>),
    Icon(Icon),
    Resattr(u16,Option<Vec<u8>>)
}

#[derive(Clone,Copy,Debug)]
pub struct Linbeg {
    pub from: ObjXY,
    pub to: ObjXY,
}

impl Linbeg {
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Linbeg> {
        Ok(Linbeg{
            from: (r.i32()?, r.i32()?).into(),
            to: (r.i32()?, r.i32()?).into(),
        })
    }
}

#[derive(Clone,Copy,Debug)]
pub struct Linstep {
    pub t: f64,
    pub e: f64,
}

impl Linstep {
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Linstep> {
        let w = r.u32()?;
        let hex_1p_10 = 0.0009765625; // hexfloat!(0x1p-10)
        let (t, e) =
        if w == 0xff_ff_ff_ff {
            (-1.0, -1.0)
        } else if (w & 0x80000000) == 0 {
            (w as f64 * hex_1p_10, -1.0)
        } else {
            let w2 = r.i32()?;
            ((w & !0x80000000) as f64 * hex_1p_10, if w2 < 0 { -1.0 } else { w2 as f64 * hex_1p_10 })
        };
        Ok(Linstep{ t: t, e: e })
    }
}

#[derive(Clone,Copy,Debug)]
pub struct Zoff {
    zoff: i16,
}

impl Zoff {
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Zoff> {
        Ok(Zoff{ zoff: r.i16()? })
    }
}

#[derive(Debug)]
pub enum Follow {
    Stop,
    To(u32, u16, String),
}

#[derive(Debug)]
pub enum Homing {
    New((i32, i32), u16),
    Change((i32, i32), u16),
    Delete,
}

#[derive(Debug)]
pub enum Buddy {
    Update(String, u8, u8),
    Delete,
}

#[derive(Debug)]
pub enum Icon {
    Set(u16),
    Del,
}

const OD_REM: u8 = 0;
const OD_MOVE: u8 = 1;
const OD_RES: u8 = 2;
const OD_LINBEG: u8 = 3;
const OD_LINSTEP: u8 = 4;
const OD_SPEECH: u8 = 5;
const OD_COMPOSE: u8 = 6;
const OD_ZOFF: u8 = 7;
const OD_LUMIN: u8 = 8;
const OD_AVATAR: u8 = 9;
const OD_FOLLOW: u8 = 10;
const OD_HOMING: u8 = 11;
const OD_OVERLAY: u8 = 12;
const OD_AUTH: u8 = 13;
const OD_HEALTH: u8 = 14;
const OD_BUDDY: u8 = 15;
const OD_CMPPOSE: u8 = 16;
const OD_CMPMOD: u8 = 17;
const OD_CMPEQU: u8 = 18;
const OD_ICON: u8 = 19;
const OD_RESATTR: u8 = 20;
const OD_END: u8 = 255;

impl ObjDataElemProp {
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Option<ObjDataElemProp>> {
        let t = r.u8()?;
        match t {
            OD_REM => Ok(Some(ObjDataElemProp::Rem)),
            OD_MOVE => {
                let xy = (r.i32()?, r.i32()?);
                let ia = (r.u16()? as f64 / 65536.0) * PI * 2.0;
                Ok(Some(ObjDataElemProp::Move(xy.into(), ia))) //TODO Rotation.into()
            }
            OD_RES => {
                let mut resid = r.u16()?;
                if (resid & 0x8000) != 0 {
                    resid &= !0x8000;
                    let sdt_len = r.u8()?;
                    let _sdt = {
                        let mut tmp = vec![0; sdt_len as usize];
                        r.read_exact(&mut tmp)?;
                        tmp
                    };
                }
                Ok(Some(ObjDataElemProp::Res(resid)))
            }
            OD_LINBEG => Ok(Some(ObjDataElemProp::Linbeg(Linbeg::from_buf(r)?))),
            OD_LINSTEP => Ok(Some(ObjDataElemProp::Linstep(Linstep::from_buf(r)?))),
            OD_SPEECH => {
                let zo = r.u16()?;
                let text = r.strz()?;
                Ok(Some(ObjDataElemProp::Speech(zo, text)))
            }
            OD_COMPOSE => {
                let resid = r.u16()?;
                Ok(Some(ObjDataElemProp::Compose(resid)))
            }
            OD_ZOFF => Ok(Some(ObjDataElemProp::Zoff(Zoff::from_buf(r)?))),
            OD_LUMIN => {
                let off = (r.i32()?, r.i32()?);
                let sz = r.u16()?;
                let str_ = r.u8()?;
                Ok(Some(ObjDataElemProp::Lumin(off, sz, str_)))
            }
            OD_AVATAR => {
                let mut layers = Vec::new();
                loop {
                    let layer = r.u16()?;
                    if layer == 65535 {
                        break;
                    }
                    layers.push(layer);
                }
                Ok(Some(ObjDataElemProp::Avatar(layers)))
            }
            OD_FOLLOW => {
                let oid = r.u32()?;
                if oid == 0xff_ff_ff_ff {
                    Ok(Some(ObjDataElemProp::Follow(Follow::Stop)))
                } else {
                    let xfres = r.u16()?;
                    let xfname = r.strz()?;
                    Ok(Some(ObjDataElemProp::Follow(Follow::To(oid, xfres, xfname))))
                }
            }
            OD_HOMING => {
                let oid = r.u32()?;
                match oid {
                    0xff_ff_ff_ff => Ok(Some(ObjDataElemProp::Homing(Homing::Delete))),
                    0xff_ff_ff_fe => {
                        let tgtc = (r.i32()?, r.i32()?);
                        let v = r.u16()?;
                        Ok(Some(ObjDataElemProp::Homing(Homing::Change(tgtc, v))))
                    }
                    _ => {
                        let tgtc = (r.i32()?, r.i32()?);
                        let v = r.u16()?;
                        Ok(Some(ObjDataElemProp::Homing(Homing::New(tgtc, v))))
                    }
                }
            }
            OD_OVERLAY => {
                let _olid = r.i32()?;
                let resid = r.u16()?;
                if (resid != 0xffff) && ((resid & 0x8000) != 0) {
                    let sdt_len = r.u8()? as usize;
                    let _sdt = {
                        let mut tmp = vec![0; sdt_len];
                        r.read_exact(&mut tmp)?;
                        tmp
                    };
                }
                Ok(Some(ObjDataElemProp::Overlay(resid & (!0x8000))))
            }
            OD_AUTH => {
                Ok(Some(ObjDataElemProp::Auth)) // Removed
            }
            OD_HEALTH => {
                let hp = r.u8()?;
                Ok(Some(ObjDataElemProp::Health(hp)))
            }
            OD_BUDDY => {
                let name = r.strz()?;
                // XXX FIXME C string is not like Rust string, it has \0 at the end,
                //          so this check is incorrect, I SUPPOSE.
                //          MOST PROBABLY we will crash here because 2 more readings.
                if name.is_empty() {
                    Ok(Some(ObjDataElemProp::Buddy(Buddy::Delete)))
                } else {
                    let group = r.u8()?;
                    let btype = r.u8()?;
                    Ok(Some(ObjDataElemProp::Buddy(Buddy::Update(name, group, btype))))
                }
            }
            OD_CMPPOSE => {
                let pfl = r.u8()?;
                let seq = r.u8()?;
                let ids1 = if (pfl & 2) != 0 {
                    let mut ids = Vec::new();
                    loop {
                        let mut resid = r.u16()?;
                        if resid == 65535 {
                            break;
                        }
                        if (resid & 0x8000) != 0 {
                            resid &= !0x8000;
                            let sdt_len = r.u8()? as usize;
                            let _sdt = {
                                let mut tmp = vec![0; sdt_len];
                                r.read_exact(&mut tmp)?;
                                tmp
                            };
                        }
                        ids.push(resid);
                    }
                    if !ids.is_empty() {
                        Some(ids)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let ids2 = if (pfl & 4) != 0 {
                    let mut ids = Vec::new();
                    loop {
                        let mut resid = r.u16()?;
                        if resid == 65535 {
                            break;
                        }
                        if (resid & 0x8000) != 0 {
                            resid &= !0x8000;
                            let sdt_len = r.u8()? as usize;
                            let _sdt = {
                                let mut tmp = vec![0; sdt_len];
                                r.read_exact(&mut tmp)?;
                                tmp
                            };
                        }
                        ids.push(resid);
                    }
                    let ttime = r.u8()?;
                    if !ids.is_empty() {
                        Some((Some(ids), ttime))
                    } else {
                        Some((None, ttime))
                    }
                } else {
                    None
                };
                Ok(Some(ObjDataElemProp::Cmppose(seq, ids1, ids2)))
            }
            OD_CMPMOD => {
                let mut m = Vec::new();
                loop {
                    let modif = r.u16()?;
                    if modif == 65535 {
                        break;
                    }
                    let mut ids = Vec::new();
                    loop {
                        let mut resid = r.u16()?;
                        if resid == 65535 {
                            break;
                        }
                        if (resid & 0x8000) != 0 {
                            resid &= !0x8000;
                            let sdt_len = r.u8()? as usize;
                            let _sdt = {
                                let mut tmp = vec![0; sdt_len];
                                r.read_exact(&mut tmp)?;
                                tmp
                            };
                        }
                        ids.push(resid);
                    }
                    if !ids.is_empty() {
                        m.push(ids);
                    }
                }
                let mods = if !m.is_empty() {
                    Some(m)
                } else {
                    None
                };
                Ok(Some(ObjDataElemProp::Cmpmod(mods)))
            }
            OD_CMPEQU => {
                let mut e = Vec::new();
                loop {
                    let h = r.u8()?;
                    if h == 255 {
                        break;
                    }
                    let at = r.strz()?;
                    let resid = r.u16()?;
                    let off = if (h & 0x80) != 0 {
                        let x = r.u16()?;
                        let y = r.u16()?;
                        let z = r.u16()?;
                        Some((x, y, z))
                    } else {
                        None
                    };
                    e.push((h & 0x7f, at, resid, off));
                }
                let equ = if !e.is_empty() {
                    Some(e)
                } else {
                    None
                };
                Ok(Some(ObjDataElemProp::Cmpequ(equ)))
            }
            OD_ICON => {
                let resid = r.u16()?;
                if resid == 65535 {
                    Ok(Some(ObjDataElemProp::Icon(Icon::Del)))
                } else {
                    let _ifl = r.u8()?;
                    Ok(Some(ObjDataElemProp::Icon(Icon::Set(resid))))
                }
            }
            OD_RESATTR => {
                let resid = r.u16()?;
                let len = r.u8()?;
                if len > 0 {
                    let dat = {
                        let mut tmp = vec![0; len as usize];
                        r.read_exact(&mut tmp)?;
                        tmp
                    };
                    Ok(Some(ObjDataElemProp::Resattr(resid, Some(dat))))
                } else {
                    Ok(Some(ObjDataElemProp::Resattr(resid, None)))
                }
            }
            OD_END => Ok(None),
            _ => {
                Err(anyhow!("unknown ObjDataElemProp: {}", t))
            }
        }
    }
}
