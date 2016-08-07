use proto::message_objdata::ObjData;
use proto::serialization::*;
use Error;

#[derive(Debug)]
pub struct ObjAck {
    pub obj: Vec<ObjAckElem>,
}

impl ObjAck {
    // TODO impl FromBuf for ObjAck {}
    pub fn from_buf <R:ReadBytesSac> (_: &mut R) -> Result<ObjAck,Error> {
        // TODO FIXME parse ObjAck instead of empty return
        Ok(ObjAck { obj: Vec::new() })
    }

    pub fn from_objdata(objdata: &ObjData) -> ObjAck {
        let mut objack = ObjAck { obj: Vec::new() };
        for o in &objdata.obj {
            objack.obj.push(ObjAckElem {
                id: o.id,
                frame: o.frame,
            });
        }
        objack
    }
}

#[derive(Debug)]
pub struct ObjAckElem {
    pub id: u32,
    pub frame: i32,
}
