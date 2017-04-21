use libc;
use lv2;
use std::ptr;
use websocket::Message;
use websocket::server::request::Request;
use websocket::client;
use websocket::header::WebSocketProtocol;
use websocket::stream::WebSocketStream;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::mem;

// Automatically generate `RustcDecodable` and `RustcEncodable` trait
// implementations
#[derive(RustcDecodable, RustcEncodable)]
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
    pub sender: client::Sender<WebSocketStream>,
    pub receiver: client::Receiver<WebSocketStream>,
    pub tcplistener: TcpListener,
    pub connected: bool,
}

impl yassyui {
    pub fn new() -> yassyui {

        let tcplistener = TcpListener::bind("127.0.0.1:55555").unwrap();
        // TODO: need to copy this manually into the javascript file. How
        // can this be automated?
        println!("UI listening at {}.", tcplistener.local_addr().unwrap());
        tcplistener.set_nonblocking(true).expect("Cannot set non-blocking");
        unsafe {
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
                sender: mem::uninitialized(),
                receiver: mem::uninitialized(),
                tcplistener: tcplistener,
                connected: false,
            };
            ui
        }
    }
}

pub fn client_split(s: TcpStream)
                    -> (client::Sender<WebSocketStream>, client::Receiver<WebSocketStream>) {
    let tcpstream = s;
    tcpstream.set_nonblocking(true).expect("set_nonblocking call failed");
    let wsstream = WebSocketStream::Tcp(tcpstream);
    pub struct Connection<R: Read, W: Write>(R, W);
    let connection = Connection(wsstream.try_clone().unwrap(), wsstream.try_clone().unwrap());

    let request = Request::read(connection.0, connection.1).unwrap();
    let headers = request.headers.clone(); // Keep the headers so we can check them

    request.validate().unwrap(); // Validate the request

    let mut response = request.accept(); // Form a response

    if let Some(&WebSocketProtocol(ref protocols)) = headers.get() {
        if protocols.contains(&("rust-websocket".to_string())) {
            // We have a protocol we want to use
            response.headers
                .set(WebSocketProtocol(vec!["rust-websocket".to_string()]));
        }
    }
    let mut client = response.send().unwrap(); // Send the response

    let ip = client.get_mut_sender()
        .get_mut()
        .peer_addr()
        .unwrap();

    println!("Connection from {}", ip);

    let message: Message = Message::text("Hello".to_string());
    client.send_message(&message).unwrap();

    client.split()
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
