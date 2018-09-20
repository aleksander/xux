use proto::serialization::*;
use Result;

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

    pub fn new (error: u8) -> sSess {
        sSess{ err: SessError::new(error) }
    }

    // TODO impl FromBuf for sSess {}
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<sSess> {
        Ok(sSess::new( r.u8()? ))
    }

    /*
    pub fn to_buf <W:WriteBytesSac> (&self, _: &mut W) -> Result<(), Error> {
        Err( Error{ source:"sSess.to_buf is not implemented yet", detail:None } )
    }
    */
}

#[allow(non_camel_case_types)]
#[derive(Debug,PartialEq)]
pub struct cSess {
    unknown: u16,
    proto: String,
    version: u16,
    pub login: String,
    pub cookie: Vec<u8>,
}

impl cSess {
    pub const ID: u8 = 0;

    #[cfg(feature = "salem")]
    pub fn new (login: String, cookie: Vec<u8>) -> cSess {
        cSess {
            unknown: 2,
            proto: "Salem".into(),
            version: 36,
            login: login,
            cookie: cookie,
        }
    }

    #[cfg(feature = "hafen")]
    pub fn new (login: String, cookie: Vec<u8>) -> cSess {
        cSess {
            unknown: 2,
            proto: "Hafen".into(),
            version: 17,
            login: login,
            cookie: cookie,
        }
    }

    // TODO impl FromBuf for cSess {}
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<cSess> {
        let unknown = r.u16()?;
        let proto = r.strz()?;
        let version = r.u16()?;
        let login = r.strz()?;
        let cookie_len = r.u16()?;
        let cookie = {
            let mut tmp = vec![0; cookie_len as usize];
            r.read_exact(&mut tmp)?;
            tmp
        };
        Ok(cSess{
            unknown: unknown,
            proto: proto,
            version: version,
            login: login,
            cookie: cookie,
        })
    }

    pub fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<()> {
        w.u8(Self::ID)?;
        w.u16(self.unknown)?;
        w.strz(&self.proto)?; // proto
        w.u16(self.version)?; // version
        w.strz(&self.login)?; // login
        w.u16(32)?; // cookie length
        w.write(self.cookie.as_slice())?; // cookie
        Ok(())
    }
}
