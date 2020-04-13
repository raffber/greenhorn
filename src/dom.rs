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
    pub target: Id,
    pub event_name: String,
    pub modifier_state: ModifierState,
    pub code: String,
    pub key: String,
    pub location: i32,
    pub repeat: bool,
    pub target_value: InputValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WheelEvent {
    pub target: Id,
    pub event_name: String,
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub delta_mode: i32,
    pub modifier_state: ModifierState,
    pub button: i32,
    pub buttons: i32,
    pub client: Point,
    pub offset: Point,
    pub page: Point,
    pub screen: Point,
    pub target_value: InputValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MouseEvent {
    pub target: Id,
    pub event_name: String,
    pub modifier_state: ModifierState,
    pub button: i32,
    pub buttons: i32,
    pub client: Point,
    pub offset: Point,
    pub page: Point,
    pub screen: Point,
    pub target_value: InputValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InputValue {
    Bool(bool),
    Text(String),
    Number(f64),
    NoValue,
}

impl InputValue {
    pub fn get_bool(&self) -> Option<bool> {
        if let InputValue::Bool(ret) = self {
            Some(*ret)
        } else {
            None
        }
    }

    pub fn get_text(&self) -> Option<String> {
        if let InputValue::Text(ret) = self {
            Some(ret.clone())
        } else {
            None
        }
    }

    pub fn get_number(&self) -> Option<f64> {
        if let InputValue::Number(ret) = self {
            Some(*ret)
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseEvent {
    pub target: Id,
    pub event_name: String,
    pub target_value: InputValue,
}


#[derive(Debug, Serialize, Deserialize)]
pub enum DomEvent {
    Base(BaseEvent),
    Focus(BaseEvent),
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
    Wheel(WheelEvent),
}

impl DomEvent {
    pub fn target(&self) -> Id {
        match self {
            DomEvent::Base(evt) => evt.target,
            DomEvent::Focus(evt) => evt.target,
            DomEvent::Keyboard(evt) => evt.target,
            DomEvent::Mouse(evt) => evt.target,
            DomEvent::Wheel(evt) => evt.target,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            DomEvent::Base(evt) => &evt.event_name,
            DomEvent::Focus(evt) => &evt.event_name,
            DomEvent::Keyboard(evt) => &evt.event_name,
            DomEvent::Mouse(evt) => &evt.event_name,
            DomEvent::Wheel(evt) => &evt.event_name,
        }
    }

    pub fn target_value(&self) -> &InputValue {
        match self {
            DomEvent::Base(e) => {&e.target_value},
            DomEvent::Focus(e) => {&e.target_value},
            DomEvent::Keyboard(e) => {&e.target_value},
            DomEvent::Mouse(e) => {&e.target_value},
            DomEvent::Wheel(e) => {&e.target_value},
        }

    }

    pub fn into_keyboard(self) -> Option<KeyboardEvent> {
        match self {
            DomEvent::Base(_) => None,
            DomEvent::Focus(_) => None,
            DomEvent::Keyboard(evt) => Some(evt),
            DomEvent::Mouse(_) => None,
            DomEvent::Wheel(_) => None,
        }
    }

    pub fn into_mouse(self) -> Option<MouseEvent> {
        match self {
            DomEvent::Base(_) => None,
            DomEvent::Focus(_) => None,
            DomEvent::Keyboard(_) => None,
            DomEvent::Mouse(evt) => Some(evt),
            DomEvent::Wheel(evt) => {
                Some(MouseEvent {
                    target: evt.target,
                    event_name: evt.event_name,
                    modifier_state: evt.modifier_state,
                    button: evt.button,
                    buttons: evt.buttons,
                    client: evt.client,
                    offset: evt.offset,
                    page: evt.page,
                    screen: evt.screen,
                    target_value: evt.target_value
                })
            },
        }
    }

    pub fn into_wheel(self) -> Option<WheelEvent> {
        match self {
            DomEvent::Base(_) => None,
            DomEvent::Focus(_) => None,
            DomEvent::Keyboard(_) => None,
            DomEvent::Mouse(_) => None,
            DomEvent::Wheel(evt) => Some(evt),
        }
    }
}

impl From<KeyboardEvent> for DomEvent {
    fn from(x: KeyboardEvent) -> Self {
        DomEvent::Keyboard(x)
    }
}

impl From<MouseEvent> for DomEvent {
    fn from(x: MouseEvent) -> Self {
        DomEvent::Mouse(x)
    }
}

impl From<WheelEvent> for DomEvent {
    fn from(x: WheelEvent) -> Self {
        DomEvent::Wheel(x)
    }
}
