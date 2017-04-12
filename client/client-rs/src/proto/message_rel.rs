use std::fmt;
use std::io::Cursor;
use proto::msg_list::MsgList;
use proto::serialization::*;
use std::io::Write;
use errors::*;

pub struct Rel {
    pub seq: u16,
    pub rel: Vec<RelElem>,
}

impl Rel {
    pub const ID: u8 = 1;

    pub fn new(seq: u16) -> Rel {
        Rel {
            seq: seq,
            rel: Vec::new(),
        }
    }
    pub fn append(&mut self, elem: RelElem) {
        self.rel.push(elem);
    }

    // TODO impl FromBuf for Rel {}
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Rel> {
        let seq = r.u16().chain_err(||"rel.from seq")?;
        let mut rel_vec = Vec::new();
        loop {
            let mut rel_type = r.u8().chain_err(||"rel.from type")?;
            let last = (rel_type & MORE_RELS_ATTACHED_BIT) == 0;
            let rel_buf = if !last {
                rel_type &= !MORE_RELS_ATTACHED_BIT;
                let rel_len = r.u16().chain_err(||"rel.from len")?;
                let mut tmp = vec![0; rel_len as usize];
                r.read_exact(&mut tmp).chain_err(||"rel.from buf")?;
                tmp
            } else {
                let mut tmp = Vec::new();
                r.read_to_end(&mut tmp).chain_err(||"rel.from buf2")?;
                tmp
            };
            rel_vec.push(RelElem::from_buf(rel_type, rel_buf.as_slice())?);
            if last { /*TODO get REMAINS*/ break; }
        }
        Ok(Rel {
            seq: seq,
            rel: rel_vec,
        })
    }
}

impl fmt::Debug for Rel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "REL seq={}", self.seq)?;
        for r in &self.rel {
            writeln!(f, "      {:?}", r)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum RelElem {
    NEWWDG(NewWdg),
    WDGMSG(WdgMsg),
    DSTWDG(DstWdg),
    MAPIV(MapIv),
    GLOBLOB(Vec<Glob>),
    PAGINAE(Paginae),
    RESID(ResId),
    PARTY(Party),
    SFX(Sfx),
    CATTR(Cattr),
    MUSIC(Music),
    TILES(Tiles),
    BUFF(Buff),
    SESSKEY(SessKey),
}

const NEWWDG: u8 = 0;
const WDGMSG: u8 = 1;
const DSTWDG: u8 = 2;
const MAPIV: u8 = 3;
const GLOBLOB: u8 = 4;
const PAGINAE: u8 = 5;
const RESID: u8 = 6;
const PARTY: u8 = 7;
const SFX: u8 = 8;
const CATTR: u8 = 9;
const MUSIC: u8 = 10;
const TILES: u8 = 11;
const BUFF: u8 = 12;
const SESSKEY: u8 = 13;

const GMSG_TIME: u8 = 0;
//const GMSG_ASTRO: u8 = 1; //TODO
const GMSG_LIGHT: u8 = 2;
const GMSG_SKY: u8 = 3;

const MORE_RELS_ATTACHED_BIT: u8 = 0x80;

impl RelElem {
    //TODO impl FromBuf for RelElem
    pub fn from_buf(kind: u8, buf: &[u8]) -> Result<RelElem> {
        let mut r = Cursor::new(buf);
        // XXX RemoteUI.java +53
        match kind {
            NEWWDG => {
                let id = r.u16().chain_err(||"relelem.from NEWWDG id")?;
                let name = r.strz().chain_err(||"relelem.from NEWWDG name")?;
                let parent = r.u16().chain_err(||"relelem.from NEWWDG parent")?;
                let pargs = MsgList::from_buf(&mut r).chain_err(||"relelem.from NEWWDG pargs")?;
                let cargs = MsgList::from_buf(&mut r).chain_err(||"relelem.from NEWWDG cargs")?;
                Ok(RelElem::NEWWDG(NewWdg {
                    id: id,
                    name: name,
                    parent: parent,
                    pargs: pargs,
                    cargs: cargs,
                }))
            }
            WDGMSG => {
                let id = r.u16().chain_err(||"relelem.from WDGMSG id")?;
                let name = r.strz()?;
                let args = MsgList::from_buf(&mut r)?;
                Ok(RelElem::WDGMSG(WdgMsg {
                    id: id,
                    name: name,
                    args: args,
                }))
            }
            DSTWDG => {
                let id = r.u16().chain_err(||"relelem.from DSTWDG id")?;
                Ok(RelElem::DSTWDG(DstWdg { id: id }))
            }
            MAPIV => Ok(RelElem::MAPIV(MapIv)),
            GLOBLOB => {
                let mut globs = Vec::new();
                let inc = r.u8().chain_err(||"relelem.from GLOBLOB inc")?;
                loop {
                    let t = match r.u8() {
                        Ok(b) => b,
                        Err(_) => break, //TODO check error type
                    };
                    globs.push(match t {
                        GMSG_TIME => {
                            Glob::Time {
                                time: r.i32().chain_err(||"relelem.from GLOBLOB TIME time")?,
                                season: r.u8().chain_err(||"relelem.from GLOBLOB TIME season")?,
                                inc: inc,
                            }
                        }
                        // GMSG_ASTRO =>
                        GMSG_LIGHT => {
                            Glob::Light {
                                amb: r.color().chain_err(||"relelem.from GLOBLOB LIGHT amb")?,
                                dif: r.color().chain_err(||"relelem.from GLOBLOB LIGHT dif")?,
                                spc: r.color().chain_err(||"relelem.from GLOBLOB LIGHT spc")?,
                                ang: r.i32().chain_err(||"relelem.from GLOBLOB LIGHT ang")?,
                                ele: r.i32().chain_err(||"relelem.from GLOBLOB LIGHT ele")?,
                                inc: inc,
                            }
                        }
                        GMSG_SKY => {
                            use std::u16;
                            let id1 = r.u16().chain_err(||"relelem.from GLOBLOB SKY id1")?;
                            Glob::Sky(if id1 == u16::MAX {
                                None
                            } else {
                                let id2 = r.u16().chain_err(||"relelem.from GLOBLOB SKY id2")?;
                                if id2 == u16::MAX {
                                    Some((id1, None))
                                } else {
                                    Some((id1, Some((id2, r.i32().chain_err(||"relelem.from GLOBLOB SKY id3")?))))
                                }
                            })
                        }
                        id => {
                            return Err(format!("unknown GLOBLOB type: {}", id).into())
                        }
                    });
                }
                Ok(RelElem::GLOBLOB(globs))
            }
            PAGINAE => Ok(RelElem::PAGINAE(Paginae)),
            RESID => {
                let id = r.u16().chain_err(||"relelem.from RESID id")?;
                let name = r.strz().chain_err(||"relelem.from RESID name")?;
                let ver = r.u16().chain_err(||"relelem.from RESID ver")?;
                Ok(RelElem::RESID(ResId {
                    id: id,
                    name: name,
                    ver: ver,
                }))
            }
            PARTY => Ok(RelElem::PARTY(Party)),
            SFX => Ok(RelElem::SFX(Sfx)),
            CATTR => Ok(RelElem::CATTR(Cattr)),
            MUSIC => Ok(RelElem::MUSIC(Music)),
            TILES => {
                let mut tiles = Vec::new();
                loop {
                    let id = match r.u8() {
                        Ok(b) => b,
                        Err(_) => break, //TODO check error type
                    };
                    let name = r.strz().chain_err(||"relelem.from TILES name")?;
                    let ver = r.u16().chain_err(||"relelem.from TILES ver")?;
                    tiles.push(TilesElem {
                        id: id,
                        name: name,
                        ver: ver,
                    });
                }
                Ok(RelElem::TILES(Tiles { tiles: tiles }))
            }
            BUFF => Ok(RelElem::BUFF(Buff)),
            SESSKEY => Ok(RelElem::SESSKEY(SessKey)),
            id => {
                Err(format!("unknown REL type: {}", id).into())
            }
        }
    }

    //TODO impl ToBuf for RelElem
    pub fn to_buf(&self, last: bool) -> Result<Vec<u8>> {
        let mut w = vec![];
        match *self {
            RelElem::WDGMSG(ref msg) => {
                let mut tmp = vec![];
                tmp.u16(msg.id).chain_err(||"relelem.to WDGMSG id")?; // widget ID
                tmp.strz(&msg.name).chain_err(||"relelem.to WDGMSG name")?;
                let args_buf = {
                    let mut v = Vec::new();
                    msg.args.to_buf(&mut v)?;
                    v
                };
                tmp.write(&args_buf).chain_err(||"relelem.to WDGMSG args")?;
                if last {
                    w.u8(WDGMSG).chain_err(||"relelem.to WDGMSG")?;
                } else {
                    use std::u16;
                    w.u8(WDGMSG | MORE_RELS_ATTACHED_BIT).chain_err(||"relelem.to WDGMSG+m")?;
                    if tmp.len() > u16::MAX as usize { return Err("relelem.to WDGMSG rel buf > u16.max".into()); }
                    w.u16(tmp.len() as u16).chain_err(||"relelem.to WDGMSG len")?; // rel length
                }
                w.write(&tmp).chain_err(||"relelem.to WDGMSG buf")?;

                Ok(w)
            }
            _ => {
                Err("relelem.to not implemented".into())
            }
        }
    }
}

#[derive(Debug)]
pub struct NewWdg {
    pub id: u16,
    pub name: String,
    pub parent: u16,
    pub pargs: Vec<MsgList>,
    pub cargs: Vec<MsgList>,
}

#[derive(Debug)]
pub struct WdgMsg {
    pub id: u16,
    pub name: String,
    pub args: Vec<MsgList>,
}

#[derive(Debug)]
pub struct DstWdg {
    pub id: u16,
}

#[derive(Debug)]
pub struct MapIv;

#[derive(Debug)]
pub enum Glob {
    Time {
        time: i32,
        season: u8,
        inc: u8,
    },
    Light {
        amb: (u8, u8, u8, u8), // TODO Color type
        dif: (u8, u8, u8, u8), // TODO Color type
        spc: (u8, u8, u8, u8), // TODO Color type
        ang: i32,
        ele: i32,
        inc: u8,
    },
    Sky(Option<(u16, Option<(u16, i32)>)>), // (resid1,resid2,blend)
}

#[derive(Debug)]
pub struct Paginae;

#[derive(Debug)]
pub struct ResId {
    pub id: u16,
    pub name: String,
    pub ver: u16,
}

#[derive(Debug)]
pub struct Party;

#[derive(Debug)]
pub struct Sfx;

#[derive(Debug)]
pub struct Cattr;

#[derive(Debug)]
pub struct Music;

pub struct Tiles {
    pub tiles: Vec<TilesElem>,
}

impl fmt::Debug for Tiles {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "")?;
        for tile in &self.tiles {
            writeln!(f, "      {:?}", tile)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct TilesElem {
    pub id: u8,
    pub name: String,
    pub ver: u16,
}

#[derive(Debug)]
pub struct Buff;

#[derive(Debug)]
pub struct SessKey;
