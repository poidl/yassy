
use libc;
use lv2;
use std::ptr;
use websocket::Message;
use websocket::stream::TcpStream as wsTcpStream;
use websocket::Server as wsServer;
use websocket::server::NoSslAcceptor;
use websocket::sender::Writer as wsWriter;
use websocket::receiver::Reader as wsReader;
// use std::mem;

// extern crate serde;
// extern crate serde_json;
// #[macro_use]
// extern crate serde_derive;
use serde_json;
use serde_json::{Error};

// Number of ports TODO: need only control ports here
pub const NPORTS: usize = 6;

#[derive(Serialize, Deserialize)]
pub struct Param {
    pub key: u32,
    pub value: f32,
}


#[repr(C)]
pub struct yassyui {
    pub extwidget: lv2::LV2UIExternalUIWidget,
    pub host: *const lv2::LV2UIExternalUIHost,
    pub controller: lv2::LV2UIController,
    pub write: lv2::LV2UIWriteFunction,
    pub showing: bool,
    // TODO: there is only one pair of sender and receiver, i.e. one connection
    // per plugin instance. If e.g. a second browser tab connects, it will
    // work but render the first browser tab unresponsive. Change this?
    pub sender: Option<wsWriter<wsTcpStream>>,
    pub receiver: Option<wsReader<wsTcpStream>>,
    pub server: Option<wsServer<NoSslAcceptor>>,
    pub instantiation_params: [f32; NPORTS]
}

impl yassyui {
    pub fn new() -> Result<yassyui, &'static str> {

        let result = wsServer::bind("127.0.0.1:55555");
        // If binding fails, instantiation must fail softly and return 
        // ptr_null(). That will happen if e.g.  the address is already 
        // in use. Consider this:
        // A user instantiates a UI, forgets to open the browser tab 
        // (-> listener keeps listening), and then instantiates a second UI.
        // On some hosts, no error message is displayed 
        // except in the console. If the user then connects, they will connect
        // to the first plugin although they expected to connected to the 
        // second one (whose instantiation failed but they don't know). 
        // Read this: http://stackoverflow.com/questions/43535480/how-can-i-reuse-a-server-side-tcp-endpoint-for-multiple-consumers
        // TODO: improve this
        match result {
            Ok(server) => {

                println!("UI listening at {}.", server.local_addr().unwrap());
                server.set_nonblocking(true).expect("Cannot set non-blocking");
                // unsafe {
                    let ui = yassyui {
                        extwidget: lv2::LV2UIExternalUIWidget {
                            // Why "None"? Nullable function pointers. See
                            // https://doc.rust-lang.org/book/ffi.html
                            // https://mail.mozilla.org/pipermail/rust-dev/2014-September/011200.html
                            run: None,
                            show: None,
                            hide: None,
                        },
                        host: ptr::null(),
                        controller: lv2::LV2UIController(ptr::null()),
                        write: None,
                        showing: false,
                        // TODO: is it possible to use Option() here?
                        sender: None,
                        receiver: None,
                        server: Some(server),
                        instantiation_params: [0f32; NPORTS]
                    };
                    Ok(ui)
                // }
            }
            _ => {
                Err("YASSYUI ERROR: BINDING FAILED (ADDRESS IN USE?)")
            }
        }
    }
}

pub fn on_ws_receive(write: lv2::LV2UIWriteFunction,
                     controller: lv2::LV2UIController,
                     param: &Param) {

    if let Some(ref func) = write {
        (*func)(controller,
                param.key,
                4,
                0,
                &param.value as &f32 as *const f32 as *const libc::c_void);
    }
    // println!("f: {}", f);
}

pub fn send_param(sender: &mut wsWriter<wsTcpStream>, param: &Param) {
    let encoded = serde_json::to_string(&param).unwrap();
    let message: Message = Message::text(encoded);
    match sender.send_message(&message) {
        Ok(_) => {},
        Err(e) => {
            println!("Send Loop: {:?}", e);
            let _ = sender.send_message(&Message::close());
        }
    }
}

pub fn param_from_string(mess: &String) -> Result<Param, Error> {
    match serde_json::from_str(mess) {
        Ok(v) => {
            let p: Param = v;
            Ok(p)
        }
        Err(err) => Err(err)
    }
}

