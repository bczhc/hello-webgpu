use std::env;

pub fn set_up_logger() {
    unsafe {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
}