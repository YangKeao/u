extern crate u;

use std::sync::Arc;
use std::sync::RwLock;
use u::Application;
use u::Window;

fn main() {
    let point_list = Arc::new(RwLock::new(vec![]));

    u::init();
    let application = u::create_application(u::Backend::X11);
    application.create_window(400, 400);

    let rp = point_list.clone();
    application.add_event_listener(Box::new(move |application, ev| match ev {
        u::Event::KeyPress(key_press_event) => match key_press_event.detail {
            65 => {
                let window = application.get_window(key_press_event.window_id);
                window.polygon(
                    &rp.read().unwrap(),
                    u::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                    },
                );

                rp.write().unwrap().clear();
                window.flush();
            }
            54 => {
                application.create_window(100, 100);
            }
            _ => {}
        },
        _ => {}
    }));
    application.add_event_listener(Box::new(|application, ev| match ev {
        u::Event::CloseNotify(close_notify_event) => {
            application.destroy_window(close_notify_event.window_id);
        }
        _ => {}
    }));

    let p = point_list.clone();
    application.add_event_listener(Box::new(move |application, ev| match ev {
        u::Event::ButtonPress(button_press_event) => {
            p.write().unwrap().push(button_press_event.cursor_position);

            let window = application.get_window(button_press_event.window_id);
            window.draw_text(button_press_event.cursor_position, u::Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            }, 50, "serif", "Hello World");
            window.flush();
        }
        _ => {}
    }));
    application.main_loop();
    return;
}
