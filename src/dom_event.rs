use crate::Id;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum DomEvent {
    Empty(),
    RawEvent(),
}

impl DomEvent {
    pub fn id(&self) -> Id {
        unimplemented!()
    }
}
