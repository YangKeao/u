extern crate u;

fn main() {
    u::init();
    let mut application = u::create_application();
    application.create_window(400, 400);
    application.create_window(200, 400);
    application.create_window(200, 600);

    application.main_loop();
    return;
}
