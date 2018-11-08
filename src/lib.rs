extern crate chrono;
extern crate libc;

use std::ffi::CString;
use std::cell::RefCell;

mod core;

thread_local! {
    static U_CORE: RefCell<core::UCore> = RefCell::new(core::UCore::new());
}

#[no_mangle]
pub extern fn schatten_process_key(key_val: u8, modifiers: u8) {
    U_CORE.with(|u_core: &RefCell<core::UCore>| {
        u_core.borrow_mut().process_key(key_val, modifiers);
    })
}

#[no_mangle]
pub extern fn schatten_get_pre_edit() -> *mut libc::c_char {
    U_CORE.with(|u_core: &RefCell<core::UCore>| {
        let pre_edit = u_core.borrow().render_pre_edit();
        let c_str_pre_edit = CString::new(pre_edit).unwrap();
        c_str_pre_edit.into_raw()
    })
}

#[no_mangle]
pub extern fn schatten_get_whether_hide() -> u8 {
    U_CORE.with(|u_core: &RefCell<core::UCore>| {
        u_core.borrow().get_hide() as u8
    })
}

#[no_mangle]
pub extern fn schatten_get_whether_should_commit() -> u8 {
    U_CORE.with(|u_core: &RefCell<core::UCore>| {
        u_core.borrow().should_commit() as u8
    })
}

#[no_mangle]
pub extern fn schatten_commit() -> *mut libc::c_char {
    U_CORE.with(|u_core: &RefCell<core::UCore>| {
        let commit_msg = u_core.borrow_mut().commit();
        let c_str_commit_msg = CString::new(commit_msg).unwrap();
        c_str_commit_msg.into_raw()
    })
}