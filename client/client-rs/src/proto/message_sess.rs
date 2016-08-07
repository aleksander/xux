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

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct cSess {
    pub login: String,
    pub cookie: Vec<u8>,
}

impl cSess {
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
}
