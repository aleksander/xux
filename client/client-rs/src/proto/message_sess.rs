use proto::serialization::*;
use Error;

#[derive(Debug)]
pub enum SessError {
    OK,
    AUTH,
    BUSY,
    CONN,
    PVER,
    EXPR,
    UNKNOWN(u8),
}

impl SessError {
    pub fn new(t: u8) -> SessError {
        match t {
            0 => SessError::OK,
            1 => SessError::AUTH,
            2 => SessError::BUSY,
            3 => SessError::CONN,
            4 => SessError::PVER,
            5 => SessError::EXPR,
            _ => SessError::UNKNOWN(t),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct sSess {
    pub err: SessError,
}

impl sSess {
    pub const ID: u8 = 0;

    // TODO impl FromBuf for sSess {}
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<sSess,Error> {
        Ok(sSess{ err: SessError::new(r.u8()?) })
    }

    /*
    pub fn to_buf <W:WriteBytesSac> (&self, _: &mut W) -> Result<(), Error> {
        Err( Error{ source:"sSess.to_buf is not implemented yet", detail:None } )
    }
    */
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct cSess {
    pub login: String,
    pub cookie: Vec<u8>,
}

impl cSess {
    pub const ID: u8 = 0;

    // TODO impl FromBuf for cSess {}
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<cSess,Error> {
        let /*unknown*/ _ = r.u16()?;
        let /*proto*/ _ = r.strz()?;
        let /*version*/ _ = r.u16()?;
        let login = r.strz()?;
        let cookie_len = r.u16()?;
        let cookie = {
            let mut tmp = vec![0; cookie_len as usize];
            r.read_exact(&mut tmp)?;
            tmp
        };
        Ok(cSess {
            login: login,
            cookie: cookie,
        })
    }

    pub fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<(), Error> {
        w.u8(Self::ID)?;
        w.u16(2)?; // unknown
        w.write("Salem".as_bytes())?; // proto
        w.u8(0)?;
        w.u16(36)?; // version
        w.write(self.login.as_bytes())?; // login
        w.u8(0)?;
        w.u16(32)?; // cookie length
        w.write(self.cookie.as_slice())?; // cookie
        Ok(())
    }
}
