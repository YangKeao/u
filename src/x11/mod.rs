extern crate xcb;

use log::*;

pub struct X11Application {
    connection: xcb::Connection,
    screen_num: i32,
    windows: std::collections::HashMap<u32, X11Window>,

}

pub struct X11Window {
    id: u32,
    foreground: u32,
}

impl X11Application {
    fn borrow_connection(&self) -> &xcb::Connection {
        &self.connection
    }
}

impl super::Application for X11Application {
    type Window = X11Window;
    type WindowIdentifier = u32;
    fn new() -> Self {
        let (connection, screen_num) = xcb::Connection::connect(None).unwrap();

        return X11Application {
            connection,
            screen_num,
            windows: std::collections::HashMap::new(),
        };
    }
    fn create_window(&mut self, width: u16, height: u16) -> u32 {
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
        xcb::map_window(&self.connection, window_id);

        self.windows.insert(
            window_id,
            X11Window {
                id: window_id,
                foreground,
            },
        );
        self.connection.flush();

        return window_id;
    }
    fn main_loop(&mut self) {
        loop {
            let event = self.connection.wait_for_event();
            match event {
                None => {
                    break;
                }
                Some(event) => {
                    let r = event.response_type() & !0x80;
                    match r {
                        xcb::EXPOSE => {
                            self.connection.flush();
                        }
                        xcb::KEY_PRESS => {
                            let key_press: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&event) };
                            trace!("Key '{}' pressed", key_press.detail());
                        }
                        xcb::KEY_RELEASE => {
                            let key_release: &xcb::KeyReleaseEvent =
                                unsafe { xcb::cast_event(&event) };
                            trace!("Key '{}' released", key_release.detail());
                        }
                        xcb::BUTTON_PRESS => {
                            let button_press: &xcb::ButtonPressEvent =
                                unsafe { xcb::cast_event(&event) };
                            trace!("Button '{}' pressed", button_press.detail());
                        }
                        xcb::BUTTON_RELEASE => {
                            let button_release: &xcb::ButtonPressEvent =
                                unsafe { xcb::cast_event(&event) };
                            trace!("Button '{}' released", button_release.detail());
                        }
                        xcb::MOTION_NOTIFY => {
                            let motion: &xcb::MotionNotifyEvent =
                                unsafe { xcb::cast_event(&event) };
                            trace!("Move to x:'{}', y:'{}'", motion.event_x(), motion.event_y());
                        }
                        xcb::ENTER_NOTIFY => {
                            let enter_event: &xcb::EnterNotifyEvent =
                                unsafe { xcb::cast_event(&event) };
                            trace!("Enter Window '{}'", enter_event.event());
                        }
                        xcb::LEAVE_NOTIFY => {
                            let leave_event: &xcb::LeaveNotifyEvent =
                                unsafe { xcb::cast_event(&event) };
                            trace!("Leave Window '{}'", leave_event.event());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    fn get_window(&mut self, id: u32) -> &X11Window {
        self.windows.get(&id).unwrap()
    }
}

impl super::Window for X11Window {
    type Application=X11Application;
    fn poly_point(&mut self, application: &X11Application, points: &[super::Point]) {
        for point in points {
            xcb::poly_point(application.borrow_connection(), xcb::COORD_MODE_ORIGIN as u8, self.id,
                            self.foreground, &[xcb::Point::new(point.x, point.y)]);
        }
    }
}
