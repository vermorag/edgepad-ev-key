//! Built-in media-key actions, emitted through a dedicated virtual keyboard.
use evdev::{uinput::VirtualDevice, AttributeSet, EventType, InputEvent, KeyCode};

pub const PREFIX: &str = "__edgepad_internal_key__";

pub fn key_code(name: &str) -> Option<KeyCode> {
    Some(match name {
        "KEY_VOLUMEUP" => KeyCode::KEY_VOLUMEUP,
        "KEY_VOLUMEDOWN" => KeyCode::KEY_VOLUMEDOWN,
        "KEY_MUTE" => KeyCode::KEY_MUTE,
        "KEY_BRIGHTNESSUP" => KeyCode::KEY_BRIGHTNESSUP,
        "KEY_BRIGHTNESSDOWN" => KeyCode::KEY_BRIGHTNESSDOWN,
        "KEY_PLAYPAUSE" => KeyCode::KEY_PLAYPAUSE,
        "KEY_NEXTSONG" => KeyCode::KEY_NEXTSONG,
        "KEY_PREVIOUSSONG" => KeyCode::KEY_PREVIOUSSONG,
        _ => return None,
    })
}

pub fn native_key_argv(name: &str) -> Result<Vec<String>, String> {
    if key_code(name).is_none() {
        return Err(format!("unsupported media key {name:?}"));
    }
    Ok(vec![PREFIX.to_owned(), name.to_owned()])
}

pub fn key_action(argv: &[String]) -> Option<KeyCode> {
    match argv {
        [prefix, name] if prefix == PREFIX => key_code(name),
        _ => None,
    }
}

#[derive(Default)]
pub struct MediaKeys {
    device: Option<VirtualDevice>,
}

impl MediaKeys {
    pub fn press(&mut self, key: KeyCode) -> Result<(), String> {
        if self.device.is_none() {
            let mut keys = AttributeSet::<KeyCode>::new();
            for name in [
                "KEY_VOLUMEUP", "KEY_VOLUMEDOWN", "KEY_MUTE",
                "KEY_BRIGHTNESSUP", "KEY_BRIGHTNESSDOWN",
                "KEY_PLAYPAUSE", "KEY_NEXTSONG", "KEY_PREVIOUSSONG",
            ] {
                keys.insert(key_code(name).expect("known key"));
            }
            self.device = Some(VirtualDevice::builder()
                .map_err(|e| format!("open uinput: {e}"))?
                .name("edgepad media keys")
                .with_keys(&keys).map_err(|e| format!("configure uinput: {e}"))?
                .build().map_err(|e| format!("create media keyboard: {e}"))?);
        }
        let dev = self.device.as_mut().expect("initialized keyboard");
        dev.emit(&[InputEvent::new(EventType::KEY, key.0, 1)])
            .map_err(|e| format!("key down: {e}"))?;
        dev.emit(&[InputEvent::new(EventType::KEY, key.0, 0)])
            .map_err(|e| format!("key up: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn media_key_names_are_validated() {
        for key in ["KEY_VOLUMEUP", "KEY_VOLUMEDOWN", "KEY_MUTE",
            "KEY_BRIGHTNESSUP", "KEY_BRIGHTNESSDOWN", "KEY_PLAYPAUSE",
            "KEY_NEXTSONG", "KEY_PREVIOUSSONG"] {
            let argv = native_key_argv(key).unwrap();
            assert_eq!(key_action(&argv), key_code(key));
        }
        assert!(native_key_argv("KEY_A").is_err());
    }
}
