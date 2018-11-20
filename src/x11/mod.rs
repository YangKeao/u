extern crate helper_macro;
extern crate xcb;

use super::*;
use log::*;

pub struct X11Application {
    connection: xcb::Connection,
    screen_num: i32,
    windows: std::cell::RefCell<std::collections::HashMap<u32, X11Window>>,
    event_listeners: std::cell::RefCell<Vec<Box<dyn Fn(&X11Application, Event<u32>) -> ()>>>
}

#[derive(Copy, Clone)]
pub struct X11Window {
    id: u32,
    foreground: u32,
}

impl X11Application {
    fn borrow_connection(&self) -> &xcb::Connection {
        &self.connection
    }
}

impl X11Application {
    fn get_atom(&self, name: &str) -> xcb::Atom {
        let cookie = xcb::intern_atom(&self.connection, true, name);
        let reply = cookie.get_reply().unwrap();
        reply.atom()
    }
}

impl Application for X11Application {
    type Window = X11Window;
    type WindowIdentifier = u32;
    fn new() -> Self {
        let (connection, screen_num) = xcb::Connection::connect(None).unwrap();

        return X11Application {
            connection,
            screen_num,
            windows: std::cell::RefCell::new(std::collections::HashMap::new()),
            event_listeners: std::cell::RefCell::new(vec![]),
        };
    }
    fn create_window(&self, width: u16, height: u16) -> u32 {
        let setup = self.connection.get_setup();
        let screen = setup.roots().nth(self.screen_num as usize).unwrap();

        let foreground = self.connection.generate_id();
        xcb::create_gc(
            &self.connection,
            foreground,
            screen.root(),
            &[
                (xcb::GC_FOREGROUND, screen.black_pixel()),
                (xcb::GC_GRAPHICS_EXPOSURES, 0),
            ],
        );

        let window_id = self.connection.generate_id();
        xcb::create_window(
            &self.connection,
            xcb::COPY_FROM_PARENT as u8,
            window_id,
            screen.root(),
            0,
            0,
            width,
            height,
            0,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            screen.root_visual(),
            &[
                (xcb::CW_BACK_PIXEL, screen.white_pixel()),
                (
                    xcb::CW_EVENT_MASK,
                    xcb::EVENT_MASK_EXPOSURE
                        | xcb::EVENT_MASK_KEY_PRESS
                        | xcb::EVENT_MASK_KEY_RELEASE
                        | xcb::EVENT_MASK_BUTTON_PRESS
                        | xcb::EVENT_MASK_BUTTON_RELEASE
                        | xcb::EVENT_MASK_POINTER_MOTION
                        | xcb::EVENT_MASK_BUTTON_MOTION
                        | xcb::EVENT_MASK_BUTTON_1_MOTION
                        | xcb::EVENT_MASK_BUTTON_2_MOTION
                        | xcb::EVENT_MASK_BUTTON_3_MOTION
                        | xcb::EVENT_MASK_BUTTON_4_MOTION
                        | xcb::EVENT_MASK_BUTTON_5_MOTION
                        | xcb::EVENT_MASK_ENTER_WINDOW
                        | xcb::EVENT_MASK_LEAVE_WINDOW,
                ),
            ],
        );
        trace!("Create Window '{}'", window_id);

        // These cookies are used for get DELETE_WINDOW event.
        // See more in ftp://www.x.org/pub/X11R7.7/doc/man/man3/xcb_change_property.3.xhtml
        // And handle close window example: https://marc.info/?l=freedesktop-xcb&m=129381953404497
        xcb::change_property(
            &self.connection,
            xcb::PROP_MODE_REPLACE as u8,
            window_id,
            self.get_atom("WM_PROTOCOLS"),
            4,
            32,
            &[self.get_atom("WM_DELETE_WINDOW")],
        );
        xcb::map_window(&self.connection, window_id);

        self.windows.borrow_mut().insert(
            window_id,
            X11Window {
                id: window_id,
                foreground,
            },
        );
        self.connection.flush();

        return window_id;
    }
    fn main_loop(&self) {
        loop {
            let event = self.connection.wait_for_event();
            match event {
                None => {
                    warn!("None Event received");
                }
                Some(event) => {
                    let r = event.response_type() & !0x80;
                    match r {
                        xcb::EXPOSE => {
                            self.trigger_event(Event::Expose(Expose {}));
                            trace!("Event EXPOSE triggered");
                        }
                        xcb::KEY_PRESS => {
                            self.trigger_event(Event::KeyPress(KeyPress {}));
                            trace!("Event KEY_PRESS triggered");
                        }
                        xcb::KEY_RELEASE => {
                            self.trigger_event(Event::KeyRelease(KeyRelease {}));
                            trace!("Event KEY_RELEASE triggered");
                        }
                        xcb::BUTTON_PRESS => {
                            self.trigger_event(Event::ButtonPress(ButtonPress {}));
                            trace!("Event BUTTON_PRESS triggered");
                        }
                        xcb::BUTTON_RELEASE => {
                            self.trigger_event(Event::ButtonRelease(ButtonRelease {}));
                            trace!("Event BUTTON_RELEASE triggered");
                        }
                        xcb::MOTION_NOTIFY => {
                            self.trigger_event(Event::MotionNotify(MotionNotify {}));
                            trace!("Event MOTION_NOTIFY triggered");
                        }
                        xcb::ENTER_NOTIFY => {
                            self.trigger_event(Event::EnterNotify(EnterNotify {}));
                            trace!("Event ENTER_NOTIFY triggered");
                        }
                        xcb::LEAVE_NOTIFY => {
                            self.trigger_event(Event::LeaveNotify(LeaveNotify {}));
                            trace!("Event LEAVE_NOTIFY triggered");
                        }
                        xcb::CLIENT_MESSAGE => {
                            let client_message : &xcb::ClientMessageEvent = unsafe {
                                xcb::cast_event(&event)
                            };
                            if client_message.data().data32()[0] == self.get_atom("WM_DELETE_WINDOW") {
                                self.trigger_event(Event::CloseNotify(CloseNotify {
                                    window_id: client_message.window()
                                }));
                                trace!("Event CLOSE_NOTIFY triggered");
                            } else {
                                trace!("Unhandled Client Message");
                            }
                        }
                        _ => {
                            warn!("Unhandled Event");
                        }
                    }
                }
            }
        }
    }
    fn get_window(&self, id: u32) -> X11Window {
        *self.windows.borrow().get(&id).unwrap()
    }
    fn flush(&self) -> bool {
        self.connection.flush()
    }

    fn add_event_listener(&self, handler: Box<Fn(&Self, Event<u32>) -> ()>) {
        self.event_listeners.borrow_mut().push(handler)
    }
    fn destroy_window(&self, window_id: u32) {
        xcb::destroy_window(&self.connection, window_id);
        self.flush();
        self.windows.borrow_mut().remove(&window_id);
        info!("Window {} destroyed", window_id);
    }
    fn trigger_event(&self, event: Event<u32>) {
        for handler in self.event_listeners.borrow().iter() {
            handler(self, event);
        }
    }
}

impl Window for X11Window {
    type Application = X11Application;
    fn poly_point(&mut self, application: &X11Application, points: &[Point]) {
        for point in points.iter() {
            xcb::poly_point(
                application.borrow_connection(),
                xcb::COORD_MODE_ORIGIN as u8,
                self.id,
                self.foreground,
                &[xcb::Point::new(point.x, point.y)],
            );
        }
    }
}
