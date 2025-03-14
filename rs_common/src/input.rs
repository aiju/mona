use bitvec::prelude::*;

#[derive(Clone, Debug)]
pub enum InputEvent {
    KeyDown(Key),
    KeyUp(Key),
    RelMouse(i32, i32),
    AbsMouse(i32, i32),
    MouseButtonDown(MouseButton),
    MouseButtonUp(MouseButton),
}

#[derive(Clone, Copy, Debug)]
pub enum MouseButton {
    Left = 0,
    Middle = 1,
    Right = 2,
}

pub trait InputSource {
    fn poll_event(&mut self) -> Option<InputEvent>;
}

pub struct NullInputSource;

impl InputSource for NullInputSource {
    fn poll_event(&mut self) -> Option<InputEvent> {
        None
    }
}

#[derive(Default, Clone)]
pub struct InputState {
    active_keys: BitArr!(for 256),
    mouse_buttons: BitArr!(for 3),
    mouse_pos: [i32; 2],
}

impl InputState {
    pub fn update(&mut self, event: InputEvent) {
        match event {
            InputEvent::KeyDown(key) => {
                self.active_keys.set(key as usize, true);
            }
            InputEvent::KeyUp(key) => {
                self.active_keys.set(key as usize, false);
            }
            InputEvent::MouseButtonDown(btn) => {
                self.mouse_buttons.set(btn as usize, true);
            }
            InputEvent::MouseButtonUp(btn) => {
                self.mouse_buttons.set(btn as usize, false);
            }
            InputEvent::AbsMouse(x, y) => self.mouse_pos = [x, y],
            InputEvent::RelMouse(x, y) => {
                self.mouse_pos = [self.mouse_pos[0] + x, self.mouse_pos[1] + y]
            }
        }
    }
    pub fn is_key_down(&self, key: Key) -> bool {
        *self.active_keys.get(key as usize).unwrap()
    }
    pub fn iter_down_keys(&self) -> impl Iterator<Item = Key> {
        self.active_keys
            .iter_ones()
            .map(|code| unsafe { std::mem::transmute::<u8, Key>(code as u8) })
    }
    pub fn is_button_down(&self, btn: MouseButton) -> bool {
        *self.mouse_buttons.get(btn as usize).unwrap()
    }
    pub fn iter_down_buttons(&self) -> impl Iterator<Item = MouseButton> {
        self.mouse_buttons
            .iter_ones()
            .map(|code| unsafe { std::mem::transmute::<u8, MouseButton>(code as u8) })
    }
    pub fn mouse_x(&self) -> i32 {
        self.mouse_pos[0]
    }
    pub fn mouse_y(&self) -> i32 {
        self.mouse_pos[1]
    }
}

impl std::fmt::Debug for InputState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputState")
            .field("active_keys", &self.iter_down_keys().collect::<Vec<_>>())
            .field("mouse_pos", &self.mouse_pos)
            .field(
                "mouse_buttons",
                &self.iter_down_buttons().collect::<Vec<_>>(),
            )
            .finish()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Key {
    Backquote,
    Backslash,
    BracketLeft,
    BracketRight,
    Comma,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Equal,
    IntlBackslash,
    IntlRo,
    IntlYen,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Minus,
    Period,
    Quote,
    Semicolon,
    Slash,
    AltLeft,
    AltRight,
    Backspace,
    CapsLock,
    ContextMenu,
    ControlLeft,
    ControlRight,
    Enter,
    MetaLeft,
    MetaRight,
    ShiftLeft,
    ShiftRight,
    Space,
    Tab,
    Delete,
    End,
    Help,
    Home,
    Insert,
    PageDown,
    PageUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Fn,
    FnLock,
    PrintScreen,
    ScrollLock,
    Pause,
}
