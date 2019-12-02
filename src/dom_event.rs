use crate::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModifierState {
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub meta_key: bool,
    pub shift_key: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyboardEvent {
    pub modifier_state: ModifierState,
    pub code: String,
    pub key: String,
    pub location: i32,
    pub repeat: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WheelEvent {
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub delta_mode: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MouseEvent {
    pub modifier_state: ModifierState,
    pub button: i32,
    pub buttons: i32,
    pub client: Point,
    pub movement: Point,
    pub offset: Point,
    pub page: Point,
    pub screen: Point,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DomEvent {
    Base(Id),
    Focus(Id),
    Keyboard(Id, KeyboardEvent),
    Mouse(Id, MouseEvent),
    Wheel(Id, WheelEvent),
}

impl DomEvent {
    pub fn target(&self) -> Id {
        match self {
            DomEvent::Base(id) => *id,
            DomEvent::Focus(id) => *id,
            DomEvent::Keyboard(id, _) => *id,
            DomEvent::Mouse(id, _) => *id,
            DomEvent::Wheel(id, _) => *id,
        }
    }

    fn into_keyboard(self) -> Option<KeyboardEvent> {
        match self {
            DomEvent::Base(_) => None,
            DomEvent::Focus(_) => None,
            DomEvent::Keyboard(_, evt) => Some(evt),
            DomEvent::Mouse(_, _) => None,
            DomEvent::Wheel(_, _) => None,
        }
    }

    fn into_mouse(self) -> Option<MouseEvent> {
        match self {
            DomEvent::Base(_) => None,
            DomEvent::Focus(_) => None,
            DomEvent::Keyboard(_, _) => None,
            DomEvent::Mouse(_, evt) => Some(evt),
            DomEvent::Wheel(_, _) => None,
        }
    }

    fn into_wheel(self) -> Option<WheelEvent> {
        match self {
            DomEvent::Base(_) => None,
            DomEvent::Focus(_) => None,
            DomEvent::Keyboard(_, _) => None,
            DomEvent::Mouse(_, _) => None,
            DomEvent::Wheel(_, evt) => Some(evt),
        }
    }
}
