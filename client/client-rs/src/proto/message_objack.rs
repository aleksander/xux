use proto::message_objdata::ObjData;

#[derive(Debug)]
pub struct ObjAck {
    pub obj: Vec<ObjAckElem>,
}

impl ObjAck {
    pub fn new(objdata: &ObjData) -> ObjAck {
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
