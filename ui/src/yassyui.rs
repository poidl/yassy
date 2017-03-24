use libc;
use lv2;
use std::ptr;
use std::thread;
use std::sync::mpsc;
use websocket::{Message, Sender, Receiver};

use websocket::server::request::Request;
use websocket::client;
use websocket::message::Type;
use websocket::header::WebSocketProtocol;
use websocket::stream::WebSocketStream;
use std::io::{Read, Write};
use std::net::TcpListener;

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

    pub fn connect(&mut self,
                   write_function: lv2::LV2UIWriteFunction,
                   controller: lv2::LV2UIController) {
        let tcplistener = TcpListener::bind("127.0.0.1:2794").unwrap();
        println!("Yassy plugin is blocking. To connect, open the file ui/client/yassyclient.html with a web browser.");
        let result = tcplistener.accept();
        match result {
            Ok(s) => {
                let tcpstream = s.0;
                let wsstream = WebSocketStream::Tcp(tcpstream);
                pub struct Connection<R: Read, W: Write>(R, W);
                let connection = Connection(wsstream.try_clone().unwrap(),
                                            wsstream.try_clone().unwrap());

                // port_event() -> param_as_message_to_sendloop() -> send_loop() ->
                // browser.
                // browser -> receive_loop() -> write_function()
                // browser -> receive_loop() -> send_loop() (for "Close" and "Ping")
                let (tx1_to_send, rx_sendloop) = mpsc::channel();
                // there are 2 producers (transmitters) for the send_loop consumer
                // (receiver). One forwards parameter values which have been translated
                // to a Message, one redirects "Close" and "Ping" signals received
                // from the browser back to the browser (through the send_loop).
                let tx2_to_send = tx1_to_send.clone();
                // send parameter values from lv2::LV2UIDescriptor.port_event()
                // to param_as_message_to_sendloop() to convert it to a message
                let (tx_2, rx_param) = mpsc::channel();
                self.sender = tx_2;

                // receive parameter values, translate it to a Message and send to
                // send_loop
                thread::spawn(move || param_as_message_to_sendloop(tx1_to_send, rx_param));

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

                let (mut sender, mut receiver) = client.split();

                // send to browser
                thread::spawn(move || send_loop(&mut sender, rx_sendloop));

                // following line works around calling on_ws_receive()
                // with raw pointer (raw opinters are not "send")
                // TODO: dangerous?
                unsafe {
                    let ctrl = &*(controller as *const i64);

                    // receive from browser
                    thread::spawn(move || {
                        receive_loop(tx2_to_send, &mut receiver, write_function, ctrl)
                    });
                }
            }
            _ => println!("error"),
        };
    }
}

fn param_as_message_to_sendloop(tx: mpsc::Sender<Message>, rx: mpsc::Receiver<Param>) {
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
        tx.send(message).unwrap();
    }
}

fn send_loop(txws: &mut client::Sender<WebSocketStream>, rx: mpsc::Receiver<Message>) {
    loop {
        // Send loop
        println!("send loop");
        let message: Message = match rx.recv() {
            Ok(m) => m,
            Err(e) => {
                println!("Send Loop: {:?}", e);
                return;
            }
        };
        match message.opcode {
            Type::Close => {
                // If it's a close message, just send it and then return.
                let _ = txws.send_message(&message);
                return;
            }
            _ => (),
        }
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
fn receive_loop(tx: mpsc::Sender<Message>,
                rxws: &mut client::Receiver<WebSocketStream>,
                write_function: lv2::LV2UIWriteFunction,
                ctrl: &i64) {
    // Loop over incoming ws messages
    for message in rxws.incoming_messages() {

        let message: Message = message.unwrap();

        match message.opcode {
            // TODO: do the right thing here
            Type::Close => {
                // Got a close message, so send a close message and return
                let _ = tx.send(Message::close());
                return;
            }
            Type::Ping => {
                match tx.send(Message::pong(message.payload)) {
                    // Send a pong in response
                    Ok(()) => (),
                    Err(e) => {
                        println!("Receive Loop: {:?}", e);
                        return;
                    }
                }
            }
            // Say what we received
            _ => {
                let vecu8 = message.payload.into_owned();
                let mess = String::from_utf8(vecu8).unwrap();
                println!("message: {}", mess);
                let res = json::decode(&mess);
                // let myfloat = mess.parse::<f32>();
                // println!("Receive Loop: {:?}", myfloat);
                match res {
                    Ok(param) => {
                        on_ws_receive(write_function, ctrl, &param);
                    }
                    Err(err) => println!("Err: {}", err),
                }
            }
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
