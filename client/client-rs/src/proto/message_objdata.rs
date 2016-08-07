use std::fmt;
use std::result::Result;
use std::fmt::Formatter;
use Error;
use proto::serialization::*;

pub struct ObjData {
    pub obj: Vec<ObjDataElem>,
}

impl ObjData {
    // TODO impl FromBuf for ObjData {}
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<ObjData,Error> {
        let mut obj = Vec::new();
        loop {
            let fl = match r.u8() {
                Ok(b) => b,
                Err(_) => {
                    break;
                }
            };
            let id = r.u32()?;
            let frame = r.i32()?;
            let mut prop = Vec::new();
            while let Some(p) = ObjDataElemProp::from_buf(r)? {
                prop.push(p)
            }
            obj.push(ObjDataElem {
                fl: fl,
                id: id,
                frame: frame,
                prop: prop,
            });
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

#[allow(non_camel_case_types)]
#[derive(Debug)]
// TODO replace with plain struct variants
pub enum ObjDataElemProp {
    odREM,
    odMOVE((i32, i32), u16),
    odRES(u16),
    odLINBEG((i32, i32), (i32, i32), i32),
    odLINSTEP(i32),
    odSPEECH(u16, String),
    odCOMPOSE(u16),
    odDRAWOFF((i32, i32)),
    odLUMIN((i32, i32), u16, u8),
    odAVATAR(Vec<u16>),
    odFOLLOW(odFOLLOW),
    odHOMING(odHOMING),
    odOVERLAY(u16),
    odAUTH,
    odHEALTH(u8),
    odBUDDY(odBUDDY),
    odCMPPOSE(u8, Option<Vec<u16>>, Option<(Option<Vec<u16>>, u8)>),
    odCMPMOD(Option<Vec<Vec<u16>>>),
    odCMPEQU(Option<Vec<(u8, String, u16, Option<(u16, u16, u16)>)>>),
    odICON(odICON),
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum odFOLLOW {
    Stop,
    To(u32, u16, String),
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum odHOMING {
    New((i32, i32), u16),
    Change((i32, i32), u16),
    Delete,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum odBUDDY {
    Update(String, u8, u8),
    Delete,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum odICON {
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
const OD_DRAWOFF: u8 = 7;
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
const OD_END: u8 = 255;

impl ObjDataElemProp {
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Option<ObjDataElemProp>, Error> {
        let t = r.u8()?;
        match t {
            OD_REM => Ok(Some(ObjDataElemProp::odREM)),
            OD_MOVE => {
                let xy = (r.i32()?, r.i32()?);
                let ia = r.u16()?;
                Ok(Some(ObjDataElemProp::odMOVE(xy, ia)))
            }
            OD_RES => {
                let mut resid = r.u16()?;
                if (resid & 0x8000) != 0 {
                    resid &= !0x8000;
                    let sdt_len = r.u8().unwrap();
                    let /*sdt*/ _ = {
                        let mut tmp = vec![0; sdt_len as usize];
                        r.read_exact(&mut tmp).unwrap();
                        tmp
                    };
                }
                Ok(Some(ObjDataElemProp::odRES(resid)))
            }
            OD_LINBEG => {
                let s = (r.i32()?, r.i32()?);
                let t = (r.i32()?, r.i32()?);
                let c = r.i32()?;
                Ok(Some(ObjDataElemProp::odLINBEG(s, t, c)))
            }
            OD_LINSTEP => {
                let l = r.i32()?;
                Ok(Some(ObjDataElemProp::odLINSTEP(l)))
            }
            OD_SPEECH => {
                let zo = r.u16()?;
                let text = r.strz()?;
                Ok(Some(ObjDataElemProp::odSPEECH(zo, text)))
            }
            OD_COMPOSE => {
                let resid = r.u16()?;
                Ok(Some(ObjDataElemProp::odCOMPOSE(resid)))
            }
            OD_DRAWOFF => {
                let off = (r.i32()?, r.i32()?);
                Ok(Some(ObjDataElemProp::odDRAWOFF(off)))
            }
            OD_LUMIN => {
                let off = (r.i32()?, r.i32()?);
                let sz = r.u16()?;
                let str_ = r.u8()?;
                Ok(Some(ObjDataElemProp::odLUMIN(off, sz, str_)))
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
                Ok(Some(ObjDataElemProp::odAVATAR(layers)))
            }
            OD_FOLLOW => {
                let oid = r.u32()?;
                if oid == 0xff_ff_ff_ff {
                    Ok(Some(ObjDataElemProp::odFOLLOW(odFOLLOW::Stop)))
                } else {
                    let xfres = r.u16()?;
                    let xfname = r.strz()?;
                    Ok(Some(ObjDataElemProp::odFOLLOW(odFOLLOW::To(oid, xfres, xfname))))
                }
            }
            OD_HOMING => {
                let oid = r.u32()?;
                match oid {
                    0xff_ff_ff_ff => Ok(Some(ObjDataElemProp::odHOMING(odHOMING::Delete))),
                    0xff_ff_ff_fe => {
                        let tgtc = (r.i32()?, r.i32()?);
                        let v = r.u16()?;
                        Ok(Some(ObjDataElemProp::odHOMING(odHOMING::Change(tgtc, v))))
                    }
                    _ => {
                        let tgtc = (r.i32()?, r.i32()?);
                        let v = r.u16()?;
                        Ok(Some(ObjDataElemProp::odHOMING(odHOMING::New(tgtc, v))))
                    }
                }
            }
            OD_OVERLAY => {
                let /*olid*/ _ = r.i32()?;
                let resid = r.u16()?;
                if (resid != 0xffff) && ((resid & 0x8000) != 0) {
                    let sdt_len = r.u8()? as usize;
                    let /*sdt*/ _ = {
                        let mut tmp = vec![0; sdt_len];
                        r.read_exact(&mut tmp).unwrap();
                        tmp
                    };
                }
                Ok(Some(ObjDataElemProp::odOVERLAY(resid & (!0x8000))))
            }
            OD_AUTH => {
                Ok(Some(ObjDataElemProp::odAUTH)) // Removed
            }
            OD_HEALTH => {
                let hp = r.u8()?;
                Ok(Some(ObjDataElemProp::odHEALTH(hp)))
            }
            OD_BUDDY => {
                let name = r.strz()?;
                // XXX FIXME C string is not like Rust string, it has \0 at the end,
                //          so this check is incorrect, I SUPPOSE.
                //          MOST PROBABLY we will crash here because 2 more readings.
                if name.is_empty() {
                    Ok(Some(ObjDataElemProp::odBUDDY(odBUDDY::Delete)))
                } else {
                    let group = r.u8()?;
                    let btype = r.u8()?;
                    Ok(Some(ObjDataElemProp::odBUDDY(odBUDDY::Update(name, group, btype))))
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
                            let /*sdt*/ _ = {
                                let mut tmp = vec![0; sdt_len];
                                r.read_exact(&mut tmp).unwrap();
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
                            let /*sdt*/ _ = {
                                let mut tmp = vec![0; sdt_len];
                                r.read_exact(&mut tmp).unwrap();
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
                Ok(Some(ObjDataElemProp::odCMPPOSE(seq, ids1, ids2)))
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
                        let resid = r.u16()?;
                        if resid == 65535 {
                            break;
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
                Ok(Some(ObjDataElemProp::odCMPMOD(mods)))
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
                Ok(Some(ObjDataElemProp::odCMPEQU(equ)))
            }
            OD_ICON => {
                let resid = r.u16()?;
                if resid == 65535 {
                    Ok(Some(ObjDataElemProp::odICON(odICON::Del)))
                } else {
                    let /*ifl*/ _ = r.u8()?;
                    Ok(Some(ObjDataElemProp::odICON(odICON::Set(resid))))
                }
            }
            OD_END => Ok(None),
            _ => {
                Ok(None) /*TODO return error*/
            }
        }
    }
}
