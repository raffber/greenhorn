use crate::Id;
use std::collections::HashSet;

pub struct Dag {

}


impl Dag {
    pub fn new() -> Self {
        todo!()
    }

    pub fn get_children(&self, id: &Id) -> HashSet<Id> {
        todo!()
    }

    pub fn find_common_ancestor(&self, ids: &HashSet<Id>) -> Id {
        todo!()
    }

    pub fn root(&self) -> Id {
        todo!()
    }
}

impl Clone for Dag {
    fn clone(&self) -> Self {
        unimplemented!()
    }
}
