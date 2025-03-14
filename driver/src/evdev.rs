use std::sync::mpsc::{self, Receiver, Sender};

use evdev::{Device, EventSummary, KeyCode, RelativeAxisCode};
use rs_common::input::{self, *};

pub struct EvdevSource {
    receiver: Receiver<InputEvent>,
}

impl EvdevSource {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        for (dev_path, dev) in evdev::enumerate() {
            let reader = EvdevReader {
                dev_path,
                sender: sender.clone(),
            };
            std::thread::spawn(move || reader.run(dev));
        }
        EvdevSource { receiver }
    }
}

impl input::InputSource for EvdevSource {
    fn poll_event(&mut self) -> Option<InputEvent> {
        self.receiver.try_recv().ok()
    }
}

struct EvdevReader {
    dev_path: std::path::PathBuf,
    sender: Sender<InputEvent>,
}

#[derive(Debug)]
enum EvdevError {
    SendError(mpsc::SendError<InputEvent>),
    IoError(std::io::Error),
}

impl From<mpsc::SendError<InputEvent>> for EvdevError {
    fn from(value: mpsc::SendError<InputEvent>) -> Self {
        EvdevError::SendError(value)
    }
}

impl From<std::io::Error> for EvdevError {
    fn from(value: std::io::Error) -> Self {
        EvdevError::IoError(value)
    }
}

impl EvdevReader {
    fn process(&mut self, event: evdev::InputEvent) -> Result<(), mpsc::SendError<InputEvent>> {
        match event.destructure() {
            EventSummary::Key(_, key_code, value) => match (map_key(key_code), value) {
                (EvdevKey::Key(key), 0) => self.sender.send(InputEvent::KeyUp(key))?,
                (EvdevKey::Key(key), 1) => self.sender.send(InputEvent::KeyDown(key))?,
                (EvdevKey::MouseButton(btn), 0) => self.sender.send(InputEvent::MouseButtonUp(btn))?,
                (EvdevKey::MouseButton(btn), 1) => self.sender.send(InputEvent::MouseButtonDown(btn))?,
                _ => {}
            },
            EventSummary::RelativeAxis(_, code, value) => match code {
                RelativeAxisCode::REL_X => self.sender.send(InputEvent::RelMouse(value, 0))?,
                RelativeAxisCode::REL_Y => self.sender.send(InputEvent::RelMouse(0, value))?,
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }
    fn run_err(&mut self, mut dev: Device) -> Result<(), EvdevError> {
        dev.grab()?;
        loop {
            for event in dev.fetch_events()? {
                self.process(event)?;
            }
        }
    }
    fn run(mut self, dev: Device) {
        if let Err(err) = self.run_err(dev) {
            eprintln!("input device error: {:?}: {:?}", self.dev_path, err);
        }
    }
}

pub enum EvdevKey {
    Unknown,
    Key(Key),
    MouseButton(MouseButton),
}

fn map_key(code: KeyCode) -> EvdevKey {
    match code {
        KeyCode::KEY_ESC => EvdevKey::Key(Key::Escape),
        KeyCode::KEY_1 => EvdevKey::Key(Key::Digit1),
        KeyCode::KEY_2 => EvdevKey::Key(Key::Digit2),
        KeyCode::KEY_3 => EvdevKey::Key(Key::Digit3),
        KeyCode::KEY_4 => EvdevKey::Key(Key::Digit4),
        KeyCode::KEY_5 => EvdevKey::Key(Key::Digit5),
        KeyCode::KEY_6 => EvdevKey::Key(Key::Digit6),
        KeyCode::KEY_7 => EvdevKey::Key(Key::Digit7),
        KeyCode::KEY_8 => EvdevKey::Key(Key::Digit8),
        KeyCode::KEY_9 => EvdevKey::Key(Key::Digit9),
        KeyCode::KEY_0 => EvdevKey::Key(Key::Digit0),
        KeyCode::KEY_MINUS => EvdevKey::Key(Key::Minus),
        KeyCode::KEY_EQUAL => EvdevKey::Key(Key::Equal),
        KeyCode::KEY_BACKSPACE => EvdevKey::Key(Key::Backspace),
        KeyCode::KEY_TAB => EvdevKey::Key(Key::Tab),
        KeyCode::KEY_Q => EvdevKey::Key(Key::KeyQ),
        KeyCode::KEY_W => EvdevKey::Key(Key::KeyW),
        KeyCode::KEY_E => EvdevKey::Key(Key::KeyE),
        KeyCode::KEY_R => EvdevKey::Key(Key::KeyR),
        KeyCode::KEY_T => EvdevKey::Key(Key::KeyT),
        KeyCode::KEY_Y => EvdevKey::Key(Key::KeyY),
        KeyCode::KEY_U => EvdevKey::Key(Key::KeyU),
        KeyCode::KEY_I => EvdevKey::Key(Key::KeyI),
        KeyCode::KEY_O => EvdevKey::Key(Key::KeyO),
        KeyCode::KEY_P => EvdevKey::Key(Key::KeyP),
        KeyCode::KEY_LEFTBRACE => EvdevKey::Key(Key::BracketLeft),
        KeyCode::KEY_RIGHTBRACE => EvdevKey::Key(Key::BracketRight),
        KeyCode::KEY_ENTER => EvdevKey::Key(Key::Enter),
        KeyCode::KEY_LEFTCTRL => EvdevKey::Key(Key::ControlLeft),
        KeyCode::KEY_A => EvdevKey::Key(Key::KeyA),
        KeyCode::KEY_S => EvdevKey::Key(Key::KeyS),
        KeyCode::KEY_D => EvdevKey::Key(Key::KeyD),
        KeyCode::KEY_F => EvdevKey::Key(Key::KeyF),
        KeyCode::KEY_G => EvdevKey::Key(Key::KeyG),
        KeyCode::KEY_H => EvdevKey::Key(Key::KeyH),
        KeyCode::KEY_J => EvdevKey::Key(Key::KeyJ),
        KeyCode::KEY_K => EvdevKey::Key(Key::KeyK),
        KeyCode::KEY_L => EvdevKey::Key(Key::KeyL),
        KeyCode::KEY_SEMICOLON => EvdevKey::Key(Key::Semicolon),
        KeyCode::KEY_APOSTROPHE => EvdevKey::Key(Key::Quote),
        KeyCode::KEY_GRAVE => EvdevKey::Key(Key::Backquote),
        KeyCode::KEY_LEFTSHIFT => EvdevKey::Key(Key::ShiftLeft),
        KeyCode::KEY_BACKSLASH => EvdevKey::Key(Key::Backslash),
        KeyCode::KEY_Z => EvdevKey::Key(Key::KeyZ),
        KeyCode::KEY_X => EvdevKey::Key(Key::KeyX),
        KeyCode::KEY_C => EvdevKey::Key(Key::KeyC),
        KeyCode::KEY_V => EvdevKey::Key(Key::KeyV),
        KeyCode::KEY_B => EvdevKey::Key(Key::KeyB),
        KeyCode::KEY_N => EvdevKey::Key(Key::KeyN),
        KeyCode::KEY_M => EvdevKey::Key(Key::KeyM),
        KeyCode::KEY_COMMA => EvdevKey::Key(Key::Comma),
        KeyCode::KEY_DOT => EvdevKey::Key(Key::Period),
        KeyCode::KEY_SLASH => EvdevKey::Key(Key::Slash),
        KeyCode::KEY_RIGHTSHIFT => EvdevKey::Key(Key::ShiftRight),
        KeyCode::KEY_LEFTALT => EvdevKey::Key(Key::AltLeft),
        KeyCode::KEY_SPACE => EvdevKey::Key(Key::Space),
        KeyCode::KEY_CAPSLOCK => EvdevKey::Key(Key::CapsLock),
        KeyCode::KEY_F1 => EvdevKey::Key(Key::F1),
        KeyCode::KEY_F2 => EvdevKey::Key(Key::F2),
        KeyCode::KEY_F3 => EvdevKey::Key(Key::F3),
        KeyCode::KEY_F4 => EvdevKey::Key(Key::F4),
        KeyCode::KEY_F5 => EvdevKey::Key(Key::F5),
        KeyCode::KEY_F6 => EvdevKey::Key(Key::F6),
        KeyCode::KEY_F7 => EvdevKey::Key(Key::F7),
        KeyCode::KEY_F8 => EvdevKey::Key(Key::F8),
        KeyCode::KEY_F9 => EvdevKey::Key(Key::F9),
        KeyCode::KEY_F10 => EvdevKey::Key(Key::F10),
        KeyCode::KEY_SCROLLLOCK => EvdevKey::Key(Key::ScrollLock),
        KeyCode::KEY_ZENKAKUHANKAKU => EvdevKey::Key(Key::Backquote),
        KeyCode::KEY_102ND => EvdevKey::Key(Key::Backslash),
        KeyCode::KEY_F11 => EvdevKey::Key(Key::F11),
        KeyCode::KEY_F12 => EvdevKey::Key(Key::F12),
        KeyCode::KEY_RO => EvdevKey::Key(Key::IntlRo),
        KeyCode::KEY_RIGHTCTRL => EvdevKey::Key(Key::ControlRight),
        KeyCode::KEY_SYSRQ => EvdevKey::Key(Key::PrintScreen),
        KeyCode::KEY_RIGHTALT => EvdevKey::Key(Key::AltRight),
        KeyCode::KEY_HOME => EvdevKey::Key(Key::Home),
        KeyCode::KEY_UP => EvdevKey::Key(Key::ArrowUp),
        KeyCode::KEY_PAGEUP => EvdevKey::Key(Key::PageUp),
        KeyCode::KEY_LEFT => EvdevKey::Key(Key::ArrowLeft),
        KeyCode::KEY_RIGHT => EvdevKey::Key(Key::ArrowRight),
        KeyCode::KEY_END => EvdevKey::Key(Key::End),
        KeyCode::KEY_DOWN => EvdevKey::Key(Key::ArrowDown),
        KeyCode::KEY_PAGEDOWN => EvdevKey::Key(Key::PageDown),
        KeyCode::KEY_INSERT => EvdevKey::Key(Key::Insert),
        KeyCode::KEY_DELETE => EvdevKey::Key(Key::Delete),
        KeyCode::KEY_PAUSE => EvdevKey::Key(Key::Pause),
        KeyCode::KEY_LEFTMETA => EvdevKey::Key(Key::MetaLeft),
        KeyCode::KEY_RIGHTMETA => EvdevKey::Key(Key::MetaRight),
        KeyCode::BTN_LEFT => EvdevKey::MouseButton(MouseButton::Left),
        KeyCode::BTN_MIDDLE => EvdevKey::MouseButton(MouseButton::Middle),
        KeyCode::BTN_RIGHT => EvdevKey::MouseButton(MouseButton::Right),
        _ => EvdevKey::Unknown,
    }
}
