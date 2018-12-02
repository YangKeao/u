#![feature(proc_macro_hygiene)]
mod x11;
use std::sync::Arc;

#[derive(Copy, Clone)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Copy, Clone)]
pub struct Position {
    pub x: i16,
    pub y: i16,
}

pub enum Backend {
    X11,
    WAYLAND,
}

#[derive(Copy, Clone)]
pub struct Expose {}

#[derive(Copy, Clone)]
pub struct KeyPress<WindowIdentifier> {
    pub window_id: WindowIdentifier,
    pub cursor_position: Position,
    pub detail: u8,
}

#[derive(Copy, Clone)]
pub struct KeyRelease {}

#[derive(Copy, Clone)]
pub struct ButtonPress<WindowIdentifier> {
    pub window_id: WindowIdentifier,
    pub cursor_position: Position,
    pub detail: u8,
}

#[derive(Copy, Clone)]
pub struct ButtonRelease {}

#[derive(Copy, Clone)]
pub struct MotionNotify {}

#[derive(Copy, Clone)]
pub struct EnterNotify {}

#[derive(Copy, Clone)]
pub struct LeaveNotify {}

#[derive(Copy, Clone)]
pub struct CloseNotify<WindowIdentifier> {
    pub window_id: WindowIdentifier,
}

#[derive(Copy, Clone)]
pub enum Event<WindowIdentifier> {
    Expose(Expose),
    KeyPress(KeyPress<WindowIdentifier>),
    KeyRelease(KeyRelease),
    ButtonPress(ButtonPress<WindowIdentifier>),
    ButtonRelease(ButtonRelease),
    MotionNotify(MotionNotify),
    EnterNotify(EnterNotify),
    LeaveNotify(LeaveNotify),
    CloseNotify(CloseNotify<WindowIdentifier>),
}

pub fn init() {
    env_logger::init();
}

pub fn create_application(backend: Backend) -> impl Application {
    match backend {
        Backend::X11 => {
            return x11::X11Application::new();
        }
        _ => panic!("Unsupported Backend"),
    }
}

pub trait Application {
    type Window: Window;
    type WindowIdentifier;
    fn new() -> Self;
    fn create_window(&self, width: u16, height: u16) -> Self::WindowIdentifier;
    fn main_loop(&self);
    fn get_window(&self, id: Self::WindowIdentifier) -> Arc<Box<Self::Window>>;
    fn windows_len(&self) -> usize;
    fn quit(&self) {
        self.set_should_quit(true);
    }
    fn set_should_quit(&self, should_quit: bool);
    fn flush(&self) -> bool;
    fn add_event_listener(&self, handler: Box<Fn(&Self, Event<Self::WindowIdentifier>) -> ()>);
    fn trigger_event(&self, event: Event<Self::WindowIdentifier>);
    fn destroy_window(&self, window_id: Self::WindowIdentifier);
}

pub trait Window {
    fn polygon(&self, points: &[Position], color: Color);
    fn draw_text(&self, position: Position, color: Color, font_size: i32, font_family: &str, content: &str);
    fn flush(&self);
}
