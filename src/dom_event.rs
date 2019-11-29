use crate::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum DomEvent {
    Empty(),
    RawEvent(),
}

impl DomEvent {
    pub fn id(&self) -> Id {
        unimplemented!()
    }
}
