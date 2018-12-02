use super::*;
use cairo::prelude::SurfaceExt;
use cairo::XCBSurface;
use log::*;
use std::sync::Arc;
use pango::prelude::*;

pub struct X11Application {
    connection: Arc<xcb::Connection>,
    screen_num: i32,
    windows: std::cell::RefCell<std::collections::HashMap<u32, Arc<Box<X11Window>>>>,
    event_listeners: std::cell::RefCell<Vec<Box<dyn Fn(&X11Application, Event<u32>) -> ()>>>,
    should_quit: std::cell::Cell<bool>,
}

pub struct X11Window {
    id: u32,
    root: *const X11Application,
    cairo_surface: cairo::Surface,
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

        let app = X11Application {
            connection: Arc::new(connection),
            screen_num,
            windows: std::cell::RefCell::new(std::collections::HashMap::new()),
            event_listeners: std::cell::RefCell::new(vec![]),
            should_quit: std::cell::Cell::new(false)
        };
        app
    }
    fn create_window(&self, width: u16, height: u16) -> u32 {
        let setup = self.connection.get_setup();
        let screen = setup.roots().nth(self.screen_num as usize).unwrap();

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
        info!("Create Window '{}'", window_id);

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
        self.connection.flush();

        self.windows.borrow_mut().insert(
            window_id,
            Arc::new(Box::new(X11Window {
                id: window_id,
                root: self,
                cairo_surface: {
                    cairo::Surface::create(
                        unsafe {
                            &cairo::XCBConnection::from_raw_full(
                                self.connection.get_raw_conn() as *mut cairo_sys::xcb_connection_t
                            )
                        },
                        &cairo::XCBDrawable(window_id),
                        &{
                            let mut visual_type = None;
                            'out: for i in setup.roots() {
                                let depth_iter = i.allowed_depths();
                                for j in depth_iter {
                                    let visuals = j.visuals();
                                    for mut v in visuals {
                                        if screen.root_visual() == v.visual_id() {
                                            visual_type = Some(unsafe {
                                                cairo::XCBVisualType::from_raw_full({
                                                    let visual_ptr = (&mut v.base)
                                                        as *mut xcb::ffi::xcb_visualtype_t;
                                                    visual_ptr as *mut cairo_sys::xcb_visualtype_t
                                                })
                                            });
                                            break 'out;
                                        }
                                    }
                                }
                            }
                            visual_type.unwrap()
                        },
                        width as i32,
                        height as i32,
                    )
                },
            })),
        );

        self.connection.flush();
        return window_id;
    }
    fn main_loop(&self) {
        loop {
            let event = self.connection.wait_for_event();
            match event {
                None => {
                    warn!("IO Error");
                }
                Some(event) => {
                    let r = event.response_type() & !0x80;
                    match r {
                        xcb::EXPOSE => {
                            self.trigger_event(Event::Expose(Expose {}));
                            trace!("Event EXPOSE triggered");
                        }
                        xcb::KEY_PRESS => {
                            let key_press_event: &xcb::KeyPressEvent =
                                unsafe { xcb::cast_event(&event) };
                            self.trigger_event(Event::KeyPress(KeyPress {
                                window_id: key_press_event.event(),
                                cursor_position: Position {
                                    x: key_press_event.event_x(),
                                    y: key_press_event.event_y(),
                                },
                                detail: key_press_event.detail(),
                            }));
                            trace!(
                                "Event KEY_PRESS triggered on WINDOW: {}",
                                key_press_event.event()
                            );
                        }
                        xcb::KEY_RELEASE => {
                            self.trigger_event(Event::KeyRelease(KeyRelease {}));
                            trace!("Event KEY_RELEASE triggered");
                        }
                        xcb::BUTTON_PRESS => {
                            let button_press_event: &xcb::ButtonPressEvent =
                                unsafe { xcb::cast_event(&event) };
                            self.trigger_event(Event::ButtonPress(ButtonPress {
                                window_id: button_press_event.event(),
                                cursor_position: Position {
                                    x: button_press_event.event_x(),
                                    y: button_press_event.event_y(),
                                },
                                detail: button_press_event.detail(),
                            }));
                            trace!(
                                "Event BUTTON_PRESS triggered on WINDOW: {}",
                                button_press_event.event()
                            );
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
                            let client_message: &xcb::ClientMessageEvent =
                                unsafe { xcb::cast_event(&event) };

                            if client_message.data().data32()[0]
                                == self.get_atom("WM_DELETE_WINDOW")
                            {
                                self.trigger_event(Event::CloseNotify(CloseNotify {
                                    window_id: client_message.window(),
                                }));
                                trace!("Event CLOSE_NOTIFY triggered");
                            } else {
                                trace!("Unhandled Client Message");
                            }
                        }
                        0 => {
                            let error_message: &xcb::GenericError =
                                unsafe { xcb::cast_event(&event) };
                            warn!(
                                "XCB Error Code: {}, Major Code: {}, Minor Code: {}",
                                error_message.error_code(),
                                unsafe { (*error_message.ptr).major_code },
                                unsafe { (*error_message.ptr).minor_code }
                            );
                        }
                        _ => {
                            warn!("Unhandled Event");
                        }
                    }
                }
            }

            if self.should_quit.get() {
                break;
            }
        }
    }
    fn get_window(&self, id: u32) -> Arc<Box<X11Window>> {
        (*self.windows.borrow().get(&id).unwrap()).clone()
    }
    fn windows_len(&self) -> usize {
        self.windows.borrow().len()
    }
    fn set_should_quit(&self, should_quit: bool) {
        self.should_quit.set(should_quit);
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

impl X11Window {
    fn get_root(&self) -> &X11Application {
        unsafe { &(*self.root) }
    }
}

impl Window for X11Window {
    fn polygon(&self, points: &[Position], color: Color) {
        if points.len() >= 2 {
            let context = cairo::Context::new(&self.cairo_surface);
            context.set_source_rgb(color.r, color.g, color.b);
            context.move_to(points[0].x as f64, points[0].y as f64);
            for i in 1..(points.len()) {
                context.line_to(points[i].x as f64, points[i].y as f64);
            }
            context.close_path();
            context.fill();
        }
    }
    fn draw_text(&self, position: Position, color: Color, font_size: i32, font_family: &str, content: &str) {
        let cr_ctx = cairo::Context::new(&self.cairo_surface);
        let pc_layout = pangocairo::functions::create_layout(&cr_ctx).unwrap();

        pc_layout.set_text(content);
        let mut font_description = pango::FontDescription::new();
        font_description.set_absolute_size((pango::SCALE * font_size) as f64);
        font_description.set_weight(pango::Weight::Bold);
        font_description.set_family(font_family);
        pc_layout.set_font_description(Some(&font_description));

        cr_ctx.set_source_rgb(color.r, color.g, color.b);
        cr_ctx.move_to(position.x as f64, position.y as f64);
        pangocairo::functions::show_layout(&cr_ctx, &pc_layout);
    }
    fn flush(&self) {
        self.cairo_surface.flush();
        self.get_root().flush();
    }
}
