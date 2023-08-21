use anyhow::Result;
use log::{debug, warn};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_long;
use std::os::raw::c_uchar;
use std::os::raw::c_uint;
use std::os::raw::c_ulong;
use std::os::raw::c_void;

pub type ProcessID = c_uint;

/// The error states that X11 can be in.
#[derive(Debug, Copy, Clone, PartialEq)]
enum XError {
    Failure,
    Success,
}

// A function that is called to handle errors, when X11 fails.
extern "C" fn handle_error_callback(
    _display_ptr: *mut x11::xlib::Display,
    error_ptr: *mut x11::xlib::XErrorEvent,
) -> c_int {
    warn!("X11 error detected.");
    if !error_ptr.is_null() {
        let xerror_data = unsafe { *error_ptr };
        debug!("X11 error data: {:?}", xerror_data);
        if xerror_data.error_code == x11::xlib::BadWindow {
            debug!("BadWindow: Window does not exist.");
            unsafe {
                X11_ERROR = XError::Failure;
            }
        }
    }

    1
}

/// The global error status of X11.
static mut X11_ERROR: XError = XError::Success;

fn get_window_id_with_focus(display_ptr: *mut x11::xlib::Display) -> c_ulong {
    let mut window_id = 0 as c_ulong;
    let mut revert_to = 0 as c_int;
    unsafe { x11::xlib::XGetInputFocus(display_ptr, &mut window_id, &mut revert_to) };
    window_id
}

#[allow(dead_code)]
fn get_top_window_id(display_ptr: *mut x11::xlib::Display, start_window_id: c_ulong) -> c_ulong {
    let mut window_id = start_window_id;
    let mut parent_window_id = start_window_id;
    let mut root_window_id = 0 as c_ulong;
    let mut child_window_ids = std::ptr::null_mut::<c_ulong>();
    let mut child_count = 0 as c_uint;

    while parent_window_id != root_window_id {
        window_id = parent_window_id;

        let status: i32 = unsafe {
            x11::xlib::XQueryTree(
                display_ptr,
                window_id,
                &mut root_window_id,
                &mut parent_window_id,
                &mut child_window_ids,
                &mut child_count,
            )
        };

        if status == (x11::xlib::Success as i32) {
            unsafe {
                x11::xlib::XFree(child_window_ids as *mut c_void);
            };
        }
    }

    window_id
}

#[allow(dead_code)]
fn list_window_properties(display_ptr: *mut x11::xlib::Display, window_id: c_ulong) -> c_int {
    // https://tronche.com/gui/x/xlib/window-information/XListProperties.html
    let mut num_prop_return = 0 as c_int;
    let properties_ptr =
        unsafe { x11::xlib::XListProperties(display_ptr, window_id, &mut num_prop_return) };
    debug!("num_prop_return: {:?}", num_prop_return);
    for i in 0..num_prop_return as isize {
        unsafe {
            let property_ptr = properties_ptr.offset(i);
            debug!("property_id: {:?}", *property_ptr);
        }
    }
    num_prop_return
}

fn get_process_id_property_id(display_ptr: *mut x11::xlib::Display) -> Result<x11::xlib::Atom> {
    // https://tronche.com/gui/x/xlib/window-information/XInternAtom.html
    let atom_name = CStr::from_bytes_with_nul(b"_NET_WM_PID\0")?;
    let atom_name_ptr = atom_name.as_ptr();
    let only_if_exists = 1 as c_int;
    let property_id: x11::xlib::Atom =
        unsafe { x11::xlib::XInternAtom(display_ptr, atom_name_ptr, only_if_exists) };
    Ok(property_id)
}

fn get_process_id_from_window_id(
    display_ptr: *mut x11::xlib::Display,
    window_id: c_ulong,
    property_id: x11::xlib::Atom,
) -> ProcessID {
    let long_offset = 0 as c_long;
    let long_length = 1 as c_long;
    let delete = x11::xlib::False as c_int;
    let req_type = x11::xlib::XA_CARDINAL;

    let mut actual_type_return = 0 as c_ulong;
    let mut actual_format_return = 0 as c_int;
    let mut nitems_return = 0 as c_ulong;
    let mut bytes_after_return = 0 as c_ulong;
    let mut prop_return_ptr: *mut c_uchar = std::ptr::null_mut();

    // https://tronche.com/gui/x/xlib/window-information/XGetWindowProperty.html
    let status: c_int = unsafe {
        x11::xlib::XGetWindowProperty(
            display_ptr,
            window_id,
            property_id,
            long_offset,
            long_length,
            delete,
            req_type,
            &mut actual_type_return,
            &mut actual_format_return,
            &mut nitems_return,
            &mut bytes_after_return,
            &mut prop_return_ptr,
        )
    };

    let mut process_id = 0;
    if status == (x11::xlib::Success as i32) {
        if actual_type_return == x11::xlib::XA_CARDINAL && actual_format_return == 32 {
            process_id = unsafe { *(prop_return_ptr as *mut ProcessID) };
        }
        unsafe { x11::xlib::XFree(prop_return_ptr as *mut c_void) };
    }

    process_id
}

fn get_process_id_from_window_tree(
    display_ptr: *mut x11::xlib::Display,
    start_window_id: c_ulong,
    property_id: x11::xlib::Atom,
) -> ProcessID {
    let mut parent_window_id = start_window_id;
    let mut root_window_id = 0 as c_ulong;
    let mut child_window_ids = std::ptr::null_mut::<c_ulong>();
    let mut child_count = 0 as c_uint;
    let mut process_id = 0;

    while parent_window_id != root_window_id {
        let window_id = parent_window_id;

        // We install a error callback to stop the program from
        // exiting when an invalid window_id is used. Instead we just
        // pretend it didn't happen.
        unsafe {
            x11::xlib::XSetErrorHandler(Some(handle_error_callback));
        }
        let status = unsafe {
            x11::xlib::XQueryTree(
                display_ptr,
                window_id,
                &mut root_window_id,
                &mut parent_window_id,
                &mut child_window_ids,
                &mut child_count,
            )
        };
        unsafe {
            x11::xlib::XSetErrorHandler(None);
        }

        unsafe {
            if X11_ERROR == XError::Failure {
                warn!("XQueryTree failed for window_id: {}", window_id);
                // Reset the global variable so we don't come here
                // again when nothing has failed.
                X11_ERROR = XError::Success;
                break;
            }
        }

        if status == (x11::xlib::Success as i32) {
            unsafe {
                x11::xlib::XFree(child_window_ids as *mut c_void);
            };
        }

        // let num_properties = list_window_properties(display_ptr, window_id);
        process_id = get_process_id_from_window_id(display_ptr, window_id, property_id);
        if process_id > 0 {
            break;
        }
    }

    process_id
}

pub fn get_active_window_process_id_from_x11() -> Result<ProcessID> {
    // Get X11 Display.
    let display_num = 0 as c_char;
    let display_ptr = unsafe { x11::xlib::XOpenDisplay(&display_num) };

    let window_id = get_window_id_with_focus(display_ptr);
    let property_id = get_process_id_property_id(display_ptr)?;
    let process_id = get_process_id_from_window_tree(display_ptr, window_id, property_id);

    // Close the X11 display.
    unsafe { x11::xlib::XCloseDisplay(display_ptr) };

    Ok(process_id)
}

pub fn get_user_idle_time_from_x11() -> c_ulong {
    let mut idle_time_sec = 0;

    // Get X11 Display.
    let display_num = 0 as c_char;
    let display_ptr = unsafe { x11::xlib::XOpenDisplay(&display_num) };

    let info_ptr = unsafe { x11::xss::XScreenSaverAllocInfo() };
    if !info_ptr.is_null() {
        let status = unsafe {
            x11::xss::XScreenSaverQueryInfo(
                display_ptr,
                x11::xlib::XDefaultRootWindow(display_ptr),
                info_ptr,
            )
        };

        if status != 0 {
            let idle_time_ms = unsafe { (*info_ptr).idle }; // milliseconds
            idle_time_sec = idle_time_ms / 1000;
            unsafe {
                x11::xlib::XFree(info_ptr as *mut c_void);
            }
        }
    }

    // Close the X11 display.
    unsafe {
        x11::xlib::XCloseDisplay(display_ptr);
    }

    idle_time_sec
}
