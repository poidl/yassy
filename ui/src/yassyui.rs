use libc;
use lv2;
use std::ptr;
use std::thread;
use std::sync::mpsc;
use websocket::{Message, Sender, Receiver};

use websocket::server::request::Request;
use websocket::client;
use websocket::header::WebSocketProtocol;
use websocket::stream::WebSocketStream;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;

use rustc_serialize::json;

// Automatically generate `RustcDecodable` and `RustcEncodable` trait
// implementations
#[derive(RustcDecodable, RustcEncodable)]
pub struct Param {
    pub key: u32,
    pub value: f32,
}


#[repr(C)]
pub struct yassyui {
    pub host: *const lv2::LV2UIExternalUIHost,
    pub controller: lv2::LV2UIController,
    pub write: lv2::LV2UIWriteFunction,
    pub extwidget: lv2::LV2UIExternalUIWidget,
    pub showing: bool,
    pub sender: mpsc::Sender<Param>,
    pub receiver: mpsc::Receiver<Param>,
}

impl yassyui {
    pub fn new() -> yassyui {
        // println!("address: {}", ipaddr);
        let (tx, rx) = mpsc::channel();
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
            controller: ptr::null(),
            write: None,
            showing: false,
            sender: tx,
            receiver: rx,
        };
        ui
    }

    // port_event() is part of the "main thread"
    // Each UI instance has a 
    // Two threads are spawned:

    // Connecting channel: 
    // THREAD 1:    param_as_message_to_sendloop()
    //           
    //    Host----------->UI-------------------------------------->Browser
    // *) Host calls port_event()
    // *) port_event() calls yassyui.sender.send(Param), which INs Param to param_as_message_to_sendloop(IN,OUT)
    // *) param_as_message_to_sendloop() encodes Param as JSON and sends it  to the browser via websocket
    //
    pub fn connect(&mut self,
                   write_function: lv2::LV2UIWriteFunction,
                   controller: lv2::LV2UIController) {
        let tcplistener = TcpListener::bind("127.0.0.1:2794").unwrap();
        println!("Yassy plugin is blocking. To connect, open the file ui/client/yassyclient.html with a web browser.");
        let result = tcplistener.accept();
        match result {
            Ok(s) => {
                let (mut sender, mut receiver) = client_split(s.0);
 
                let (tx_hostin, rx_hostin) = mpsc::channel();
                self.sender = tx_hostin;


                // receive parameter values, translate it to a Message and send to
                // send_loop
                thread::spawn(move || param_as_message_to_sendloop(&mut sender, rx_hostin));

                // send to browser
                // thread::spawn(move || send_loop(&mut sender, rx_wsout));

                // following line works around calling on_ws_receive()
                // with raw pointer (raw opinters are not "send")
                // TODO: dangerous?
                unsafe {
                    let ctrl = &*(controller as *const i64);

                    // receive from browser
                    thread::spawn(move || {
                        receive_loop( &mut receiver, write_function, ctrl)
                    });
                }
            }
            _ => println!("error"),
        };
    }
}

fn client_split(s: TcpStream) -> (client::Sender<WebSocketStream>, client::Receiver<WebSocketStream>) {
    let tcpstream = s;
    let wsstream = WebSocketStream::Tcp(tcpstream);
    pub struct Connection<R: Read, W: Write>(R, W);
    let connection = Connection(wsstream.try_clone().unwrap(),
                                wsstream.try_clone().unwrap());

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

fn param_as_message_to_sendloop(txws: &mut client::Sender<WebSocketStream>, rx: mpsc::Receiver<Param>) {
    loop {
        let param: Param = match rx.recv() {
            Ok(v) => v,
            Err(e) => {
                println!("Oeha: {:?}", e);
                return;
            }
        };
        println!("param.key: {}", param.key);
        println!("param.value: {}", param.value);
        let encoded = json::encode(&param).unwrap();
        let message: Message = Message::text(encoded);

        // tx.send(message).unwrap();
        // Send the message
        match txws.send_message(&message) {
            Ok(()) => (),
            Err(e) => {
                println!("Send Loop: {:?}", e);
                let _ = txws.send_message(&Message::close());
                return;
            }
        }
    }
}

// Receive from browser
fn receive_loop(rxws: &mut client::Receiver<WebSocketStream>,
                write_function: lv2::LV2UIWriteFunction,
                ctrl: &i64) {
    // Loop over incoming ws messages
    for message in rxws.incoming_messages() {

        let message: Message = message.unwrap();
        let vecu8 = message.payload.into_owned();
        let mess = String::from_utf8(vecu8).unwrap();
        println!("message: {}", mess);
        let res = json::decode(&mess);
        match res {
            Ok(param) => {
                on_ws_receive(write_function, ctrl, &param);
            }
            Err(err) => println!("Err: {}", err),
        }
    }
}

fn on_ws_receive(write: lv2::LV2UIWriteFunction, controller: &i64, param: &Param) {

    let ctrl = controller as *const i64 as lv2::LV2UIController;
    if let Some(ref func) = write {
        (*func)(ctrl, param.key, 4, 0, &param.value as &f32 as *const f32 as *const libc::c_void);
    }
    // println!("f: {}", f);
}
