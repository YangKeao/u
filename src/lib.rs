#![feature(proc_macro_hygiene)]
extern crate env_logger;
extern crate log;

mod x11;

pub enum Backend {
    X11,
    WAYLAND,
}

#[derive(Copy, Clone)]
pub struct Expose {}

#[derive(Copy, Clone)]
pub struct KeyPress {}

#[derive(Copy, Clone)]
pub struct KeyRelease {}

#[derive(Copy, Clone)]
pub struct ButtonPress {}

#[derive(Copy, Clone)]
pub struct ButtonRelease {}

#[derive(Copy, Clone)]
pub struct MotionNotify {}

#[derive(Copy, Clone)]
pub struct EnterNotify {}

#[derive(Copy, Clone)]
pub struct LeaveNotify {}

#[derive(Copy, Clone)]
pub enum Event {
    Expose(Expose),
    KeyPress(KeyPress),
    KeyRelease(KeyRelease),
    ButtonPress(ButtonPress),
    ButtonRelease(ButtonRelease),
    MotionNotify(MotionNotify),
    EnterNotify(EnterNotify),
    LeaveNotify(LeaveNotify),
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
    fn create_window(&mut self, width: u16, height: u16) -> Self::WindowIdentifier;
    fn main_loop(&mut self);
    fn get_window(&mut self, id: Self::WindowIdentifier) -> &Self::Window;
    fn flush(&mut self) -> bool;
    fn add_event_listener(&mut self, handler: Box<Fn(Event) -> ()>);
    fn trigger_event(&mut self, event: Event);
}

pub struct Point {
    x: i16,
    y: i16,
}

pub trait Window {
    type Application;
    fn poly_point(&mut self, application: &Self::Application, points: &[Point]);
}
