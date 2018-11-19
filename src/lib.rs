extern crate env_logger;
extern crate log;

mod x11;

pub enum Backend {
    X11,
    WAYLAND,
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
}

pub struct Point {
    x: i16,
    y: i16,
}

pub trait Window {
    type Application;
    fn poly_point(&mut self, application: &Self::Application, points: &[Point]);
}
