function serializeModifierState(evt) {
    return {
        "alt_key": evt.altKey,
        "ctrl_key": evt.ctrlKey,
        "meta_key": evt.metaKey,
        "shift_key": evt.shiftKey
    };
}

function serializePoint(x,y) {
    return {
        "x": x,
        "y": y
    };
}

function serializeMouseEvent(id, name, evt) {
    return {
        "target": {"id": id},
        "event_name": name,
        "modifier_state": serializeModifierState(evt),
        "button": evt.button,
        "buttons": evt.buttons,
        "client": serializePoint(evt.clientX, evt.clientY),
        "offset": serializePoint(evt.offsetX, evt.offsetY),
        "page": serializePoint(evt.pageX, evt.pageY),
        "screen": serializePoint(evt.screenX, evt.screenY),
        "target_value": serializeTargetValue(evt.target)
    };
}

function serializeTargetValue(target) {
    let v =  target.value;
    if (typeof v === "string") {
        return {"Text": v};
    } else if (typeof v === "boolean") {
        return {"Bool": v};
    } else if (typeof v === "number") {
        return {"Number": v};
    } else {
        return "NoValue";
    }    
}

export default function serializeEvent(id, name, evt) {
    if (evt instanceof WheelEvent) {
        let wheel =  {
            "delta_x": evt.deltaX,
            "delta_y": evt.deltaY,
            "delta_z": evt.deltaZ,
            "delta_mode": evt.deltaMode
        };
        return {
            "Wheel": { ...wheel, ...serializeMouseEvent(id, name, evt) }
        }
    } else if (evt instanceof MouseEvent) {
        return {
            "Mouse": serializeMouseEvent(id, name, evt)
        }
    } else if (evt instanceof KeyboardEvent) {
        return {
            "Keyboard": {
                "target": {"id": id},
                "event_name": name,            
                "modifier_state": serializeModifierState(evt),
                "code": evt.code,
                "key": evt.key,
                "location": evt.location,
                "repeat": evt.repeat,
                "bubble": true,
                "target_value": serializeTargetValue(evt.target)
            }
        }
    } else if (evt instanceof FocusEvent) {
        return {
            "Focus": {
                "target": {"id": id},
                "event_name": name,            
                "target_value": serializeTargetValue(evt.target)
            }
        }
    } else {
        return {
            "Base": {
                "target": {"id": id},
                "event_name": name,            
                "target_value": serializeTargetValue(evt.target)
            }
        }
    }
}
