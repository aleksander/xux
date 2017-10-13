use std::fmt;
use proto::list::List;
use proto::serialization::*;
use std::io::Write;
use errors::*;

pub struct Rels {
    pub seq: u16,
    pub rels: Vec<Rel>,
}

impl Rels {
    pub const ID: u8 = 1;
    pub const MORE_RELS_ATTACHED_BIT: u8 = 0x80;

    pub fn new(seq: u16) -> Rels {
        Rels {
            seq: seq,
            rels: Vec::new(),
        }
    }

    pub fn append(&mut self, rel: Rel) {
        self.rels.push(rel);
    }

    // TODO impl FromBuf for Rel {}
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Rels> {
        let seq = r.u16().chain_err(||"rels.from seq")?;
        let mut rels = Vec::new();
        loop {
            let mut rel_type = r.u8().chain_err(||"rels.from type")?;
            let last = (rel_type & Rels::MORE_RELS_ATTACHED_BIT) == 0;
            let rel_buf = if !last {
                rel_type &= !Rels::MORE_RELS_ATTACHED_BIT;
                let rel_len = r.u16().chain_err(||"rels.from len")?;
                let mut tmp = vec![0; rel_len as usize];
                r.read_exact(&mut tmp).chain_err(||"rels.from buf")?;
                tmp
            } else {
                let mut tmp = Vec::new();
                r.read_to_end(&mut tmp).chain_err(||"rels.from buf2")?;
                tmp
            };
            rels.push(Rel::from_buf(rel_type, rel_buf.as_slice())?);
            if last { /*TODO get REMAINS*/ break; }
        }
        Ok(Rels {
            seq: seq,
            rels: rels,
        })
    }
}

impl fmt::Debug for Rels {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "REL seq={}", self.seq)?;
        for r in &self.rels {
            writeln!(f, "      {:?}", r)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Rel {
    NEWWDG(NewWdg),
    WDGMSG(WdgMsg),
    DSTWDG(DstWdg),
    MAPIV(MapIv),
    GLOBLOB(Globs),
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

impl Rel {
    //TODO impl FromBuf for RelElem
    pub fn from_buf(kind: u8, r: &[u8]) -> Result<Rel> {
        let mut r = ::std::io::Cursor::new(r);
        // XXX RemoteUI.java +53
        match kind {
            NewWdg::ID => Ok(Rel::NEWWDG(NewWdg::from_buf(r)?)),
            WdgMsg::ID => Ok(Rel::WDGMSG(WdgMsg::from_buf(r)?)),
            DstWdg::ID => Ok(Rel::DSTWDG(DstWdg::from_buf(r)?)),
            MapIv::ID => Ok(Rel::MAPIV(MapIv)),
            Globs::ID => Ok(Rel::GLOBLOB(Globs::from_buf(&mut r)?)),
            Paginae::ID => Ok(Rel::PAGINAE(Paginae)),
            ResId::ID => Ok(Rel::RESID(ResId::from_buf(r)?)),
            Party::ID => Ok(Rel::PARTY(Party)),
            Sfx::ID => Ok(Rel::SFX(Sfx)),
            Cattr::ID => Ok(Rel::CATTR(Cattr)),
            Music::ID => Ok(Rel::MUSIC(Music)),
            Tiles::ID => Ok(Rel::TILES(Tiles::from_buf(r)?)),
            Buff::ID => Ok(Rel::BUFF(Buff)),
            SessKey::ID => Ok(Rel::SESSKEY(SessKey)),
            id => {
                Err(format!("unknown REL type: {}", id).into())
            }
        }
    }

    //TODO impl ToBuf for RelElem
    pub fn to_buf(&self, last: bool) -> Result<Vec<u8>> {
        let mut w = vec![];
        match *self {
            Rel::WDGMSG(ref msg) => {
                let mut tmp = vec![];
                tmp.u16(msg.id).chain_err(||"rel.to WDGMSG id")?; // widget ID
                tmp.strz(&msg.name).chain_err(||"rel.to WDGMSG name")?;
                let args_buf = {
                    let mut v = Vec::new();
                    msg.args.to_buf(&mut v)?;
                    v
                };
                tmp.write(&args_buf).chain_err(||"rel.to WDGMSG args")?;
                if last {
                    w.u8(WdgMsg::ID).chain_err(||"rel.to WDGMSG")?;
                } else {
                    use std::u16;
                    w.u8(WdgMsg::ID | Rels::MORE_RELS_ATTACHED_BIT).chain_err(||"rel.to WDGMSG+m")?;
                    if tmp.len() > u16::MAX as usize { return Err("rel.to WDGMSG rel buf > u16.max".into()); }
                    w.u16(tmp.len() as u16).chain_err(||"rel.to WDGMSG len")?; // rel length
                }
                w.write(&tmp).chain_err(||"rel.to WDGMSG buf")?;

                Ok(w)
            }
            _ => {
                Err("rel.to not implemented".into())
            }
        }
    }
}

#[derive(Debug)]
pub struct NewWdg {
    pub id: u16, //TODO enum WdgID(u16)
    pub name: String,
    pub parent: u16, //TODO enum WdgID(u16)
    pub pargs: Vec<List>,
    pub cargs: Vec<List>,
}

impl NewWdg {
    pub const ID: u8 = 0;

    fn from_buf <R:ReadBytesSac> (mut r: R) -> Result<NewWdg> {
        let id = r.u16().chain_err(||"NEWWDG id")?;
        let name = r.strz().chain_err(||"NEWWDG name")?;
        let parent = r.u16().chain_err(||"NEWWDG parent")?;
        let pargs = List::from_buf(&mut r).chain_err(||"NEWWDG pargs")?;
        let cargs = List::from_buf(&mut r).chain_err(||"NEWWDG cargs")?;
        Ok(NewWdg {
            id: id,
            name: name,
            parent: parent,
            pargs: pargs,
            cargs: cargs,
        })
    }
}

#[derive(Debug)]
pub struct WdgMsg {
    pub id: u16,
    pub name: String,
    pub args: Vec<List>,
}

impl WdgMsg {
    pub const ID: u8 = 1;

    pub fn new (id: u16, name: String, args: Vec<List>) -> WdgMsg {
        WdgMsg {
            id: id,
            name: name,
            args: args,
        }
    }

    fn from_buf <R:ReadBytesSac> (mut r: R) -> Result<WdgMsg> {
        let id = r.u16().chain_err(||"WDGMSG id")?;
        let name = r.strz().chain_err(||"WDGMSG name")?;
        let args = List::from_buf(&mut r).chain_err(||"WDGMSG args")?;
        Ok(WdgMsg {
            id: id,
            name: name,
            args: args,
        })
    }
}

#[derive(Debug)]
pub struct DstWdg {
    pub id: u16,
}

impl DstWdg {
    pub const ID: u8 = 2;

    fn from_buf <R:ReadBytesSac> (mut r: R) -> Result<DstWdg> {
        let id = r.u16().chain_err(||"DSTWDG id")?;
        Ok(DstWdg{ id: id })
    }
}

#[derive(Debug)]
pub struct MapIv;

impl MapIv {
    pub const ID: u8 = 3;
}

#[derive(Debug)]
pub struct Globs {
    globs: Vec<Glob>
}

impl Globs {
    pub const ID: u8 = 4;

    #[cfg(feature = "salem")]
    fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Globs> {
        let mut globs = Vec::new();
        let inc = r.u8().chain_err(||"Globs.from inc")?;
        loop {
            let t = match r.u8() {
                Ok(b) => b,
                Err(_) => break, //TODO check error type
            };
            globs.push(Glob::from_buf(r, t, inc)?);
        }
        Ok(Globs{ globs: globs })
    }

    #[cfg(feature = "hafen")]
    fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Globs> {
        let mut globs = Vec::new();
        let inc = r.u8().chain_err(||"Globs.from inc")?;
        loop {
            let t = match r.strz() {
                Ok(b) => b,
                Err(_) => break, //TODO check error type
            };
            globs.push(Glob::from_buf(r, &t, inc)?);
        }
        Ok(Globs{ globs: globs })
    }
}

#[cfg(feature = "salem")]
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

#[cfg(feature = "hafen")]
#[derive(Debug)]
pub enum Glob {
    Tm,
    Astro,
    Light,
    Sky,
    Wth,
}

impl Glob {
    #[cfg(feature = "salem")]
    const GMSG_TIME: u8 = 0;
    //const GMSG_ASTRO: u8 = 1; //TODO
    #[cfg(feature = "salem")]
    const GMSG_LIGHT: u8 = 2;
    #[cfg(feature = "salem")]
    const GMSG_SKY: u8 = 3;

    #[cfg(feature = "salem")]
    fn from_buf <R:ReadBytesSac> (r: &mut R, t: u8, inc: u8) -> Result<Glob> {
        Ok(match t {
            Self::GMSG_TIME => {
                Glob::Time {
                    time: r.i32().chain_err(||"Glob.from TIME time")?,
                    season: r.u8().chain_err(||"Glob.from TIME season")?,
                    inc: inc,
                }
            }
            // GMSG_ASTRO =>
            Self::GMSG_LIGHT => {
                Glob::Light {
                    amb: r.color().chain_err(||"Glob.from LIGHT amb")?,
                    dif: r.color().chain_err(||"Glob.from LIGHT dif")?,
                    spc: r.color().chain_err(||"Glob.from LIGHT spc")?,
                    ang: r.i32().chain_err(||"Glob.from LIGHT ang")?,
                    ele: r.i32().chain_err(||"Glob.from LIGHT ele")?,
                    inc: inc,
                }
            }
            Self::GMSG_SKY => {
                use std::u16;
                let id1 = r.u16().chain_err(||"Glob.from SKY id1")?;
                Glob::Sky(if id1 == u16::MAX {
                    None
                } else {
                    let id2 = r.u16().chain_err(||"Glob.from SKY id2")?;
                    if id2 == u16::MAX {
                        Some((id1, None))
                    } else {
                        Some((id1, Some((id2, r.i32().chain_err(||"Glob.from SKY id3")?))))
                    }
                })
            }
            id => {
                return Err(format!("unknown GLOBLOB type: {}", id).into())
            }
        })
    }

    #[cfg(feature = "hafen")]
    fn from_buf <R:ReadBytesSac> (r: &mut R, t: &str, _inc: u8) -> Result<Glob> {
        let _list = List::from_buf(r)?;
        Ok(match t {
            "tm" => {Glob::Tm}
            "astro" => {Glob::Astro}
            "light" => {Glob::Light}
            "sky" => {Glob::Sky}
            "wth" => {Glob::Wth}
            _ => {
                return Err(format!("unknown GLOBLOB type: '{:?}'", t.as_bytes()).into())
            }
        })
    }
}

#[derive(Debug)]
pub struct Paginae;

impl Paginae {
    pub const ID: u8 = 5;
}

#[derive(Debug)]
pub struct ResId {
    pub id: u16,
    pub name: String,
    pub ver: u16,
}

impl ResId {
    pub const ID: u8 = 6;

    fn from_buf <R:ReadBytesSac> (mut r: R) -> Result<ResId> {
        let id = r.u16().chain_err(||"ResId.from id")?;
        let name = r.strz().chain_err(||"ResId.from name")?;
        let ver = r.u16().chain_err(||"ResId.from ver")?;
        Ok(ResId {
            id: id,
            name: name,
            ver: ver,
        })
    }
}

#[derive(Debug)]
pub struct Party;

impl Party {
    pub const ID: u8 = 7;
}

#[derive(Debug)]
pub struct Sfx;

impl Sfx {
    pub const ID: u8 = 8;
}

#[derive(Debug)]
pub struct Cattr;

impl Cattr {
    pub const ID: u8 = 9;
}

#[derive(Debug)]
pub struct Music;

impl Music {
    pub const ID: u8 = 10;
}

pub struct Tiles {
    pub tiles: Vec<Tile>,
}

impl Tiles {
    pub const ID: u8 = 11;

    fn from_buf <R:ReadBytesSac> (mut r: R) -> Result<Tiles> {
        let mut tiles = Vec::new();
        loop {
            let id = match r.u8() {
                Ok(b) => b,
                Err(_) => break, //TODO check error type
            };
            let name = r.strz().chain_err(||"TILES name")?;
            let ver = r.u16().chain_err(||"TILES ver")?;
            tiles.push(Tile {
                id: id,
                name: name,
                ver: ver,
            });
        }
        Ok(Tiles { tiles: tiles })
    }
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

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Tile {
    pub id: u8,
    pub name: String,
    pub ver: u16,
}

#[derive(Debug)]
pub struct Buff;

impl Buff {
    pub const ID: u8 = 12;
}

#[derive(Debug)]
pub struct SessKey;

impl SessKey {
    pub const ID: u8 = 13;
}
