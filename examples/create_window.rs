extern crate u;

use u::Application;

fn main() {
    u::init();
    let application = u::create_application(u::Backend::X11);
    application.create_window(400, 400);

    application.add_event_listener(Box::new(|application, ev| match ev {
        u::Event::KeyPress(_key_press_event) => {
            application.create_window(1000, 1000);
        }
        _ => {}
    }));
    application.add_event_listener(Box::new(|application, ev| match ev {
        u::Event::CloseNotify(close_notify_event) => {
            application.destroy_window(close_notify_event.window_id);
        }
        _ => {}
    }));
    application.main_loop();
    return;
}
