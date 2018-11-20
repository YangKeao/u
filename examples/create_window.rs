extern crate u;

use u::Application;

fn main() {
    u::init();
    let application = u::create_application(u::Backend::X11);
    application.create_window(400, 400);

    application.add_event_listener(Box::new(|application, ev: u::Event| match ev {
        u::Event::KeyPress(_key_press_event) => {
            application.create_window(1000, 1000);
        }
        _ => {}
    }));
    application.main_loop();
    return;
}
