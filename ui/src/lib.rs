
extern crate libc;
extern crate lv2;
extern crate websocket;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod yassyui;
use std::mem;
use std::ffi::CStr;
use std::str;
use std::ptr;

use serde_json::{Error};
use websocket::Message;

// Credits to Hanspeter Portner for explaining how ui:UI and kx:Widget work. See
// http://lists.lv2plug.in/pipermail/devel-lv2plug.in/2016-May/001649.html

// have to define new type. Otherwise error: "cannot define inherent impl for a type outside of
// the crate where the type is defined; define and implement a trait or new type instead"
#[repr(C)]
struct Descriptor(lv2::LV2UIDescriptor);

impl Descriptor {
    pub extern "C" fn instantiate(descriptor: *const lv2::LV2UIDescriptor,
                                  _plugin_uri: *const libc::c_char,
                                  _bundle_path: *const libc::c_char,
                                  write_function: lv2::LV2UIWriteFunction,
                                  controller: lv2::LV2UIController,
                                  widget: *mut lv2::LV2UIWidget,
                                  features: *const (*const lv2::LV2Feature))
                                  -> lv2::LV2UIHandle {
        println!("host calls instantiate()");
        lv2::print_features(features);
        match yassyui::yassyui::new() {
            Ok(yas) => {

                let mut bx = Box::new(yas);

                bx.controller = controller;
                bx.write = write_function;
                let uitype = unsafe { lv2::cstring((*descriptor).uri) };
                println!("UITYPE: {}", uitype);
                if uitype == "http://example.org/yassyui#kx" {
                    println!("MAPPING FEATURE FOR: {}", uitype);
                    let featureptr = lv2::mapfeature(features,
                                                    "http://kxstudio.sf.net/ns/lv2ext/external-ui#Host");
                    match featureptr {
                        Ok(fp) => bx.host = fp as *const lv2::LV2UIExternalUIHost,
                        _ => return ptr::null_mut(),
                    }
                    bx.extwidget.run = Some(kx_run);
                    bx.extwidget.show = Some(kx_show);
                    bx.extwidget.hide = Some(kx_hide);
                    unsafe {
                        *widget = &mut bx.extwidget as *mut lv2::LV2UIExternalUIWidget as lv2::LV2UIWidget
                    }
                }

                let ptr = (&*bx as *const yassyui::yassyui) as *mut libc::c_void;
                mem::forget(bx);
                ptr
            }
            // default, e.g. if the port is already bound
            Err(e) => {
                println!("YASSYUI ERROR: INSTANTIATION FAILED: {}. REFUSE TO PANIC TO AVOID BRINGING DOWN THE HOST.", e);
                ptr::null_mut()
            }
        }
    }

    pub extern "C" fn cleanup(_handle: lv2::LV2UIHandle)     {
        println!("host calls cleanup()");
    }

    pub extern "C" fn port_event(ui: lv2::LV2UIHandle,
                                 port_index: libc::c_uint,
                                 _buffer_size: libc::c_uint,
                                 _format: libc::c_uint,
                                 buffer: *const libc::c_void) {
        println!("host calls port_event() on port_index: {}", port_index);

        unsafe {
            let buf = *(buffer as *const libc::c_float);
            println!("  buffer: {}", buf);
            let yas = ui as *mut yassyui::yassyui;
            match (*yas).sender {
                Some(ref mut sender) => {
                    // sender exists -> already connected

                    let param = yassyui::Param{key: port_index, value: buf as f32};
                    let encoded = serde_json::to_string(&param).unwrap();
                    let message: Message = Message::text(encoded);
                    match sender.send_message(&message) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Send Loop: {:?}", e);
                            let _ = sender.send_message(&Message::close());
                            return;
                        }
                    }
                }
                _ => {} 
            };
        }
    }

    pub extern "C" fn extension_data(uri: *const libc::c_char) -> *const libc::c_void {
        unsafe {
            // println!("Host calls extension_data:");
            let buf = CStr::from_ptr(uri).to_bytes();
            let s: &str = str::from_utf8(buf).unwrap();
            if s == "http://lv2plug.in/ns/extensions/ui#idleInterface" {
                return &IDLEINTERFACE as *const lv2::LV2UIIdleInterface as *const libc::c_void;
            } else if s == "http://lv2plug.in/ns/extensions/ui#showInterface" {
                return &SHOWINTERFACE as *const lv2::LV2UIShowInterface as *const libc::c_void;
            }

            ptr::null() as *const libc::c_void
        }
    }
}

static SUI: &'static [u8] = b"http://example.org/yassyui#ui\0";

static mut DESC_UI: lv2::LV2UIDescriptor = lv2::LV2UIDescriptor {
    uri: 0 as *const libc::c_char, // ptr::null() isn't const fn (yet)
    instantiate: Descriptor::instantiate,
    cleanup: Descriptor::cleanup,
    port_event: Descriptor::port_event,
    extension_data: Some(Descriptor::extension_data),
};

static SKX: &'static [u8] = b"http://example.org/yassyui#kx\0";

static mut DESC_KX: lv2::LV2UIDescriptor = lv2::LV2UIDescriptor {
    uri: 0 as *const libc::c_char, // ptr::null() isn't const fn (yet)
    instantiate: Descriptor::instantiate,
    cleanup: Descriptor::cleanup,
    port_event: Descriptor::port_event,
    extension_data: None,
};

static mut IDLEINTERFACE: lv2::LV2UIIdleInterface = lv2::LV2UIIdleInterface { idle: ui_idle };
static mut SHOWINTERFACE: lv2::LV2UIShowInterface = lv2::LV2UIShowInterface {
    show: ui_show,
    hide: ui_hide,
};

#[no_mangle]
pub extern "C" fn lv2ui_descriptor(index: i32) -> *const lv2::LV2UIDescriptor {
    // credits to ker on stackoverflow:
    // http://stackoverflow.com/questions/31334356/static-struct-with
    // -c-strings-for-lv2-plugin (duplicate) or http://stackoverflow.com/questions/
    // 25880043/creating-a-static-c-struct-containing-strings

    // Credits to Hanspeter Portner for explaining how to use ui:UI and
    // kx:Widget:
    // http://lists.lv2plug.in/pipermail/devel-lv2plug.in/2016-May/001649.html
    let ptr: *const libc::c_char;
    unsafe {
        match index {
            0 => {
                ptr = SUI.as_ptr() as *const libc::c_char;
                DESC_UI.uri = ptr;
                return &DESC_UI as *const lv2::LV2UIDescriptor;
            }
            1 => {
                ptr = SKX.as_ptr() as *const libc::c_char;
                DESC_KX.uri = ptr;
                return &DESC_KX as *const lv2::LV2UIDescriptor;
            }
            _ => return std::ptr::null(),
        }
    }
}

#[no_mangle]
pub extern "C" fn ui_idle(handle: lv2::LV2UIHandle) -> libc::c_int {
    // returns non-zero if the UI has been closed, in which case the host
    // should stop calling idle(), and can either completely destroy the UI, or
    // re-show it and resume calling idle().
    // println!("host calls idle()");
    let ui = handle as *mut yassyui::yassyui;
    unsafe {
        match (*ui).receiver {

            Some(ref mut receiver) => {
                // receiver is present -> already connected

                // Loop over 5 incoming ws messages. Will block if not
                // breaking out. If one uses no loop at all, latency is 
                // high. 
                // TODO: This will depend on the frequency with which
                // ui_idle() is called by host.
                for message in receiver.incoming_messages().take(5) {
                    match message {
                        Ok(m) => {
                            let message: Message =m;
                            let vecu8 = message.payload.into_owned();
                            let mess = String::from_utf8(vecu8).unwrap();
                            println!("Client says: {}", mess);
                            let res = param_from_string(&mess);
                            match res {
                                Ok(param) => {
                                    yassyui::on_ws_receive((*ui).write, (*ui).controller, &param);
                                }
                                Err(err) => println!("Err: {}", err),
                            }
                            
                        },
                        _ => {}
                    }
                }

            }
            _ => {
                // still disconnected

                match (*ui).server {
                    // server should exist since instantiation
                    Some(ref mut server) => {
                        // connect
                        match server.accept() {
                            Ok(wsupgrade) => {
                                wsupgrade.tcp_stream().set_nonblocking(true).expect("Cannot set non-blocking");
                                match wsupgrade.accept() {
                                    Ok(client) => {
                                        let (receiver, sender) = client.split().unwrap();
                                        (*ui).receiver = Some(receiver);
                                        (*ui).sender= Some(sender);
                                        (*ui).server= None;

                                    }
                                    _=> {}
                                }
                            }

                            _ => {}
                        }
                    }
                    _ => {
                        println!("YASSYUI ERROR: SERVER DOESN'T EXIST. REFUSE TO PANIC TO AVOID BRINGING DOWN THE HOST.");
                    }

                }

            } 
        }
        return !(*ui).showing as libc::c_int;
    }
}

fn param_from_string(mess: &String) -> Result<yassyui::Param, Error> {
    match serde_json::from_str(mess) {
        Ok(v) => {
            let p: yassyui::Param = v;
            Ok(p)
        }
        Err(err) => Err(err)
    }
}

#[no_mangle]
pub extern "C" fn ui_show(handle: lv2::LV2UIHandle) -> libc::c_int {
    // Show a window for this UI. Returns 0 on success, or anything else to
    // stop being called. on success, or anything else to stop being called.
    println!("host calls show()");
    let ui = handle as *mut yassyui::yassyui;
    unsafe {
        if (*ui).showing {
            return 0i32 as libc::c_int; // already showing
        }
        println!("   do something in show()");
        (*ui).showing = true;
        return 0i32 as libc::c_int;
    }
}

#[no_mangle]
pub extern "C" fn ui_hide(handle: lv2::LV2UIHandle) -> libc::c_int {
    // Hide the window for this UI. Returns 0 on success, or anything else to
    // stop being called. on success, or anything else to stop being called.
    println!("host calls hide()");
    let ui = handle as *mut yassyui::yassyui;
    unsafe {
        (*ui).showing = false;
    }
    return 0i32 as libc::c_int;
}

#[no_mangle]
pub extern "C" fn kx_run(exthandle: *const lv2::LV2UIExternalUIWidget) {
    // Host calls this function regulary. UI library implementing the
    // callback may do IPC or redraw the UI.
    // println!("host calls kx_run()");
    let offset = get_offset();
    unsafe {
        let uihandle = (exthandle as lv2::LV2UIHandle).offset(offset);
        let ui = uihandle as *mut yassyui::yassyui;
        if ui_idle(uihandle) == 1i32 {
            // ui_closed: Callback that plugin UI will call when UI (GUI window) is closed by user.
            // This callback will be called during execution of LV2_External_UI_Widget::run()
            // (i.e. not from background thread).

            // destructure tuple struct to access lv2_raw::LV2UIControllerRaw
            let controller_raw = (*ui).controller.0;
            ((*(*ui).host).ui_closed)(controller_raw);
            ui_hide(uihandle);
        }
    }
}

#[no_mangle]
pub extern "C" fn kx_show(exthandle: *const lv2::LV2UIExternalUIWidget) {
    println!("host calls kx_show()");
    let offset = get_offset();
    unsafe {
        let uihandle = (exthandle as lv2::LV2UIHandle).offset(offset);
        ui_show(uihandle);
    }
}

#[no_mangle]
pub extern "C" fn kx_hide(exthandle: *const lv2::LV2UIExternalUIWidget) {
    println!("host calls kx_hide()");
    let offset = get_offset();
    unsafe {
        let uihandle = (exthandle as lv2::LV2UIHandle).offset(offset);
        let ui = uihandle as *mut yassyui::yassyui;
        (*ui).showing = false;
        ui_hide(uihandle);
    }
}

fn get_offset() -> isize {
    // compute offset in bytes between struct yassyui and member extwidget.
    // needed for in the kx_* functions. AFAIK the only way to avoid this
    // would be to make sure that extwidget is always the *first* member of
    // yassyui, in which case the offset is zero
    // println!{"***** in get_offset()"}
    // let ya = yassyui::yassyui::new();
    // let uiptr = &ya as *const yassyui::yassyui as isize;
    // let extptr = &ya.extwidget as *const lv2::LV2UIExternalUIWidget as isize;
    // uiptr - extptr
    return 0 as isize
}
