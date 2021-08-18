use std::fmt;
use crate::proto::list::List;
use crate::proto::serialization::*;
use std::io::Write;
use crate::Result;
use anyhow::anyhow;
use crate::state::WdgID;

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
        let seq = r.u16()?;
        let mut rels = Vec::new();
        loop {
            let mut rel_type = r.u8()?;
            let last = (rel_type & Rels::MORE_RELS_ATTACHED_BIT) == 0;
            let rel_buf = if !last {
                rel_type &= !Rels::MORE_RELS_ATTACHED_BIT;
                let rel_len = r.u16()?;
                let mut tmp = vec![0; rel_len as usize];
                r.read_exact(&mut tmp)?;
                tmp
            } else {
                let mut tmp = Vec::new();
                r.read_to_end(&mut tmp)?;
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
    FRAGMENT(Fragment),
    ADDWDG(AddWdg),
}

impl Rel {
    //TODO impl FromBuf for RelElem
    pub fn from_buf(kind: u8, mut r: &[u8]) -> Result<Rel> {
        let r = &mut r;
        match kind {
            NewWdg::ID => Ok(Rel::NEWWDG(NewWdg::from_buf(r)?)),
            WdgMsg::ID => Ok(Rel::WDGMSG(WdgMsg::from_buf(r)?)),
            DstWdg::ID => Ok(Rel::DSTWDG(DstWdg::from_buf(r)?)),
            MapIv::ID => Ok(Rel::MAPIV(MapIv)),
            Globs::ID => Ok(Rel::GLOBLOB(Globs::from_buf(r)?)),
            Paginae::ID => Ok(Rel::PAGINAE(Paginae)),
            ResId::ID => Ok(Rel::RESID(ResId::from_buf(r)?)),
            Party::ID => Ok(Rel::PARTY(Party)),
            Sfx::ID => Ok(Rel::SFX(Sfx)),
            Cattr::ID => Ok(Rel::CATTR(Cattr)),
            Music::ID => Ok(Rel::MUSIC(Music)),
            Tiles::ID => Ok(Rel::TILES(Tiles::from_buf(r)?)),
            Buff::ID => Ok(Rel::BUFF(Buff)),
            SessKey::ID => Ok(Rel::SESSKEY(SessKey)),
            Fragment::ID => Ok(Rel::FRAGMENT(Fragment::from_buf(r)?)),
            AddWdg::ID => Ok(Rel::ADDWDG(AddWdg::from_buf(r)?)),
            id => {
                Err(anyhow!("unknown REL type: {}", id))
            }
        }
    }

    //TODO impl ToBuf for RelElem
    pub fn to_buf(&self, last: bool) -> Result<Vec<u8>> {
        let mut w = vec![];
        match *self {
            Rel::WDGMSG(ref msg) => {
                let mut tmp = vec![];
                tmp.u32(msg.id)?; // widget ID
                tmp.strz(&msg.name)?;
                let args_buf = {
                    let mut v = Vec::new();
                    msg.args.to_buf(&mut v)?;
                    v
                };
                tmp.write(&args_buf)?;
                if last {
                    w.u8(WdgMsg::ID)?;
                } else {
                    use std::u16;
                    w.u8(WdgMsg::ID | Rels::MORE_RELS_ATTACHED_BIT)?;
                    if tmp.len() > u16::MAX as usize { return Err(anyhow!("rel.to WDGMSG rel buf > u16.max")); }
                    w.u16(tmp.len() as u16)?; // rel length
                }
                w.write(&tmp)?;

                Ok(w)
            }
            ref other => {
                Err(anyhow!("rel.to is not implemented for {:?}", other))
            }
        }
    }
}

#[derive(Debug)]
pub struct NewWdg {
    pub id: WdgID, //TODO enum WdgID(u16)
    pub name: String,
    pub parent: WdgID, //TODO enum WdgID(u16)
    pub pargs: Vec<List>,
    pub cargs: Vec<List>,
}

impl NewWdg {
    pub const ID: u8 = 0;

    fn from_buf <R:ReadBytesSac> (mut r: R) -> Result<NewWdg> {
        let id = r.u32()?;
        let name = r.strz()?;
        let parent = r.u32()?;
        let pargs = List::from_buf(&mut r)?;
        let cargs = List::from_buf(&mut r)?;
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
    pub id: WdgID,
    pub name: String,
    pub args: Vec<List>,
}

impl WdgMsg {
    pub const ID: u8 = 1;

    pub fn new (id: u32, name: String, args: Vec<List>) -> WdgMsg {
        WdgMsg {
            id: id,
            name: name,
            args: args,
        }
    }

    fn from_buf <R:ReadBytesSac> (mut r: R) -> Result<WdgMsg> {
        let id = r.u32()?;
        let name = r.strz()?;
        let args = List::from_buf(&mut r)?;
        Ok(WdgMsg {
            id: id,
            name: name,
            args: args,
        })
    }
}

#[derive(Debug)]
pub struct DstWdg {
    pub id: WdgID,
}

impl DstWdg {
    pub const ID: u8 = 2;

    fn from_buf <R:ReadBytesSac> (mut r: R) -> Result<DstWdg> {
        let id = r.u32()?;
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

    fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Globs> {
        let mut globs = Vec::new();
        let inc = r.u8()?;
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

#[derive(Debug)]
pub enum Glob {
    Tm,
    Astro,
    Light,
    Sky,
    Wth,
}

impl Glob {
    fn from_buf <R:ReadBytesSac> (r: &mut R, t: &str, _inc: u8) -> Result<Glob> {
        let _list = List::from_buf(r)?;
        Ok(match t {
            "tm" => {Glob::Tm}
            "astro" => {Glob::Astro}
            "light" => {Glob::Light}
            "sky" => {Glob::Sky}
            "wth" => {Glob::Wth}
            _ => {
                return Err(anyhow!("unknown GLOBLOB type: '{:?}'", t.as_bytes()))
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
        let id = r.u16()?;
        let name = r.strz()?;
        let ver = r.u16()?;
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
    pub tiles: Vec<TileRes>,
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
            let name = r.strz()?;
            let ver = r.u16()?;
            tiles.push(TileRes {
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

#[derive(Clone,Debug)]
pub struct TileRes {
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

#[derive(Debug)]
pub enum Fragment {
    Head(u8, Vec<u8>),
    Middle(Vec<u8>),
    Tail(Vec<u8>),
}

impl Fragment {
    pub const ID: u8 = 14;

    fn from_buf <R:ReadBytesSac> (mut r: R) -> Result<Fragment> {
        let head = r.u8()?;
        let buf = {
            let mut buf = Vec::new();
            r.read_to_end(&mut buf)?;
            buf
        };
        Ok(match head {
            0x81 => {
                Fragment::Tail(buf)
            }
            0x80 => {
                Fragment::Middle(buf)
            }
            _ => {
                if head & 0x80 == 0 {
                    Fragment::Head(head, buf)
                } else {
                    return Err(anyhow!("wrong framgent type {}", head));
                }
            }
        })
    }
}

#[derive(Debug)]
pub struct AddWdg {
    pub id: WdgID, //TODO enum WdgID(u16)
    pub parent: WdgID, //TODO enum WdgID(u16)
    pub pargs: Vec<List>,
}

impl AddWdg {
    pub const ID: u8 = 15;

    fn from_buf <R:ReadBytesSac> (mut r: R) -> Result<AddWdg> {
        let id = r.u32()?;
        let parent = r.u32()?;
        let pargs = List::from_buf(&mut r)?;
        Ok(AddWdg {
            id: id,
            parent: parent,
            pargs: pargs,
        })
    }
}
