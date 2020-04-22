//! This module exposes the [DomEvent](struct.DomEvent.html) data type which is used to communicate
//! DOM events from the frontend to the backend.
//!
//! [DomEvents](struct.DomEvent.html) are the principal data type for frontend
//! to backend communication. When subscribing to a event on the DOM a handler
//! function is registered in the application runtime, which maps a
//! [DomEvent](struct.DomEvent.html) to a
//! the message type of the running [../trait.App.html].
//!
//! When the subscribed event is triggered on the frontend,
//! a [DomEvent](struct.DomEvent.html) is created, mapped with the handler function
//! and subsequently passed into the `update()` cycle of the application.
//!
use crate::Id;
use serde::{Deserialize, Serialize};


/// Defines whether a modifier is currently pressed
#[derive(Debug, Serialize, Deserialize)]
pub struct ModifierState {
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub meta_key: bool,
    pub shift_key: bool,
}

/// Mapping of the [HTML KeyboardEvent](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent)
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

/// Mapping of the [HTML WheelEvent](https://developer.mozilla.org/en-US/docs/Web/API/WheelEvent)
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

/// Maps to an (x,y) coordinate tuple for HTML MouseEvents
#[derive(Debug, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Mapping of the [HTML MouseEvent](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent)
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

/// Maps to the `value` attribute of `HTMLElement`.
///
/// In case the element has no `value` attribute or it has an unsupported type,
/// the `InputValue::NoValue` type is used.
#[derive(Debug, Serialize, Deserialize)]
pub enum InputValue {
    Bool(bool),
    Text(String),
    Number(f64),
    NoValue,
}

impl InputValue {
    /// Attempt to convert this value into a bool
    pub fn get_bool(&self) -> Option<bool> {
        if let InputValue::Bool(ret) = self {
            Some(*ret)
        } else {
            None
        }
    }

    /// Attempt to convert this value into a string
    pub fn get_text(&self) -> Option<String> {
        if let InputValue::Text(ret) = self {
            Some(ret.clone())
        } else {
            None
        }
    }

    /// Attempt to convert this value into a number
    pub fn get_number(&self) -> Option<f64> {
        if let InputValue::Number(ret) = self {
            Some(*ret)
        } else {
            None
        }
    }
}

/// Minimal data type to represent unsupported [HTML Events](https://developer.mozilla.org/en-US/docs/Web/API/Event).
#[derive(Debug, Serialize, Deserialize)]
pub struct BaseEvent {
    pub target: Id,
    pub event_name: String,
    pub target_value: InputValue,
}

/// The `DomEvent` enum maps HTML Events into a rust datatype.
///
/// `DomEvent`s are the principal form of communication between the frontend and the backend.
/// Whenever a HTML Event is triggered and the backend has subscribed to it, a message with a
/// `DomEvent` is passed into the `update()` cycle of the `App`.
#[derive(Debug, Serialize, Deserialize)]
pub enum DomEvent {
    Base(BaseEvent),
    Focus(BaseEvent),
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
    Wheel(WheelEvent),
}

impl DomEvent {
    /// Returns the `Id` of the DOM node which triggered this event
    pub fn target(&self) -> Id {
        match self {
            DomEvent::Base(evt) => evt.target,
            DomEvent::Focus(evt) => evt.target,
            DomEvent::Keyboard(evt) => evt.target,
            DomEvent::Mouse(evt) => evt.target,
            DomEvent::Wheel(evt) => evt.target,
        }
    }

    /// Returns the event name that triggered this event.
    ///
    /// The names don't use "on". Event names are e.g. "change" or "click"
    pub fn name(&self) -> &str {
        match self {
            DomEvent::Base(evt) => &evt.event_name,
            DomEvent::Focus(evt) => &evt.event_name,
            DomEvent::Keyboard(evt) => &evt.event_name,
            DomEvent::Mouse(evt) => &evt.event_name,
            DomEvent::Wheel(evt) => &evt.event_name,
        }
    }

    /// Attempts to map the `event.target.value` attribute of a primitive rust type.
    ///
    /// This is useful for [HTMLInputElements](https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement).
    pub fn target_value(&self) -> &InputValue {
        match self {
            DomEvent::Base(e) => {&e.target_value},
            DomEvent::Focus(e) => {&e.target_value},
            DomEvent::Keyboard(e) => {&e.target_value},
            DomEvent::Mouse(e) => {&e.target_value},
            DomEvent::Wheel(e) => {&e.target_value},
        }

    }

    /// Attempt to convert this type into a [KeyboardEvent](struct.KeyboardEvent.html)
    pub fn into_keyboard(self) -> Option<KeyboardEvent> {
        match self {
            DomEvent::Base(_) => None,
            DomEvent::Focus(_) => None,
            DomEvent::Keyboard(evt) => Some(evt),
            DomEvent::Mouse(_) => None,
            DomEvent::Wheel(_) => None,
        }
    }

    /// Attempt to convert this type into a [MouseEvent](struct.MouseEvent.html)
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

    /// Attempt to convert this type into a [WheelEvent](struct.WheelEvent.html)
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
