extern crate u;

use u::Application;

fn main() {
    u::init();
    let mut application = u::create_application(u::Backend::X11);
    application.create_window(400, 400);
    application.create_window(200, 400);
    application.create_window(200, 600);

    application.add_event_listener(Box::new(|ev: u::Event| {

    }));
    application.main_loop();
    return;
}
