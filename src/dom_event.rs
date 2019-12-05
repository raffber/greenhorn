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
    pub mouse_event: MouseEvent,
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
    Base(Id, String),
    Focus(Id, String),
    Keyboard(Id, String, KeyboardEvent),
    Mouse(Id, String, MouseEvent),
    Wheel(Id, String, WheelEvent),
}

impl DomEvent {
    pub fn target(&self) -> Id {
        match self {
            DomEvent::Base(id, _) => *id,
            DomEvent::Focus(id, _) => *id,
            DomEvent::Keyboard(id, _, _) => *id,
            DomEvent::Mouse(id, _, _) => *id,
            DomEvent::Wheel(id, _, _) => *id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            DomEvent::Base(_, name) => name,
            DomEvent::Focus(_, name) => name,
            DomEvent::Keyboard(_, name, _) => name,
            DomEvent::Mouse(_, name, _) => name,
            DomEvent::Wheel(_, name, _) => name,
        }
    }

    pub fn into_keyboard(self) -> Option<KeyboardEvent> {
        match self {
            DomEvent::Base(_, _) => None,
            DomEvent::Focus(_, _) => None,
            DomEvent::Keyboard(_, _, evt) => Some(evt),
            DomEvent::Mouse(_, _, _) => None,
            DomEvent::Wheel(_, _, _) => None,
        }
    }

    pub fn into_mouse(self) -> Option<MouseEvent> {
        match self {
            DomEvent::Base(_, _) => None,
            DomEvent::Focus(_, _) => None,
            DomEvent::Keyboard(_, _, _) => None,
            DomEvent::Mouse(_, _, evt) => Some(evt),
            DomEvent::Wheel(_, _, evt) => Some(evt.mouse_event),
        }
    }

    pub fn into_wheel(self) -> Option<WheelEvent> {
        match self {
            DomEvent::Base(_, _) => None,
            DomEvent::Focus(_, _) => None,
            DomEvent::Keyboard(_, _, _) => None,
            DomEvent::Mouse(_, _, _) => None,
            DomEvent::Wheel(_, _, evt) => Some(evt),
        }
    }
}
