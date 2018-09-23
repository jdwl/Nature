///! World Connection Service provider
extern crate uuid;

use nature_common::util::setup_logger;
pub use self::channels::*;
pub use self::service_type_define::*;
use std::thread::JoinHandle;

// for product and mock
lazy_static! {
    // sys biz define
    pub static ref SYS_KEY_SERIAL : String = "/serial".to_string();

    // sys context define
    pub static ref CONTEXT_TARGET_INSTANCE_ID : String = "sys.target_instance_id".to_string();
}

pub fn sys_init() -> Vec<JoinHandle<()>> {
    let _ = setup_logger();

    // read config
//    let config = world_connection::util::get_settings();

    start_receive_threads()
}

pub fn finish_threads<T>(threads: Vec<JoinHandle<T>>) {
    for t in threads {
        let _ = t.join();
    }
}

mod channels;

mod service_type_define;

