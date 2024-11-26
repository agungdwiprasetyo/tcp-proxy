use core::str;
use std::{fs, io};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use model::Address;

mod model;

fn main() {
    let source = fs::read_to_string("source.json").expect("Unable to read file");
    let addresses: Vec<Address> = serde_json::from_str(&source).expect("Unable to parse");

    let mut processes = vec![];
    for address in addresses {
        let (proxy_addr, to_addr) = (&address.source, &address.target);
        let listener = TcpListener::bind(proxy_addr).expect("Unable to bind proxy addr");
    
        println!("Proxing TCP connection from {} to {}", proxy_addr, to_addr);

        let listener_arc = Arc::new(listener);
        let listener_tx = listener_arc.try_clone().unwrap();

        processes.push(thread::spawn(
            move || handle_incomming_message(listener_tx, &address.target)
        ));
    }

    for t in processes {
        t.join().unwrap();
    }
}

fn handle_incomming_message(listener: TcpListener, to_addr: &str) {
    for incoming_stream in listener.incoming() {
        let src_stream = incoming_stream.unwrap();
        let conn_thread = TcpStream::connect(to_addr)
            .map(|dest_stream| thread::spawn(
                move || proxy_src_to_dest(src_stream, dest_stream)
            ));

        match conn_thread {
            Ok(_) => { println!("Successfully proxy to target: {}", to_addr); }
            Err(_) => { println!("Unable proxy to target: {}", to_addr); }
        }
    }
}

fn proxy_src_to_dest(src_conn: TcpStream, target_conn: TcpStream) {
    let src_arc = Arc::new(src_conn);
    let target_arc = Arc::new(target_conn);

    let (mut src_tx, mut src_rx) = (src_arc.try_clone().unwrap(), src_arc.try_clone().unwrap());
    let (mut target_tx, mut target_rx) = (target_arc.try_clone().unwrap(), target_arc.try_clone().unwrap());

    let connections = vec![
        thread::spawn(move || io::copy(&mut src_tx, &mut target_rx).unwrap()),
        thread::spawn(move || io::copy(&mut target_tx, &mut src_rx).unwrap()),
    ];

    for t in connections {
        t.join().unwrap();
    }
}
