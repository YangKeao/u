#![feature(proc_macro_hygiene)]
extern crate env_logger;
extern crate log;

mod x11;

pub enum Backend {
    X11,
    WAYLAND,
}

pub struct Expose {}

pub struct KeyPress {}

pub struct KeyRelease {}

pub struct ButtonPress {}

pub struct ButtonRelease {}

pub struct MotionNotify {}

pub struct EnterNotify {}

pub struct LeaveNotify {}

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
    fn add_event_listener<F>(&mut self, handler: F)
    where
        F: Fn(Event) -> ();
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
