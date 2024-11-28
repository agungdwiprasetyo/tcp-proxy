use core::str;
use std::io::{BufReader, Read, Write};
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::thread;

use model::Address;

mod macros;
mod model;

fn main() {
    let source = fs::read_to_string("source.json").expect("Unable to read file");
    let addresses: Vec<Address> = serde_json::from_str(&source).expect("Unable to parse");

    let mut processes = vec![];
    for address in addresses {
        let (proxy_addr, to_addr) = (&address.source, &address.target);
        let listener = TcpListener::bind(proxy_addr).expect("Unable to bind proxy addr");
    
        println!("Proxing TCP connection from {} to {}", proxy_addr, to_addr);

        processes.push(thread::spawn(
            move || handle_incomming_message(listener, &address.target)
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
    let (src_tx, src_rx) = (src_conn.try_clone().unwrap(), src_conn.try_clone().unwrap());
    let (target_tx, target_rx) = (target_conn.try_clone().unwrap(), target_conn.try_clone().unwrap());

    let src_to_target_log = format!("[{}:{}] to [{}:{}]", 
        src_rx.peer_addr().unwrap().ip().to_string(), src_rx.peer_addr().unwrap().port(), 
        target_rx.peer_addr().unwrap().ip().to_string(), target_rx.peer_addr().unwrap().port());
    let target_to_src_log = format!("[{}:{}] to [{}:{}]", 
        target_rx.peer_addr().unwrap().ip().to_string(), target_rx.peer_addr().unwrap().port(), 
        src_rx.peer_addr().unwrap().ip().to_string(), src_rx.peer_addr().unwrap().port());

    let connections = vec![
        thread::spawn(move || copy_conn(src_tx, target_rx, src_to_target_log)),
        thread::spawn(move || copy_conn(target_tx, src_rx, target_to_src_log)),
    ];

    for t in connections {
        t.join().unwrap();
    }
}

fn copy_conn(src: TcpStream, dest: TcpStream, log: String) {
    let proxy = src.try_clone().expect("Cannot clone stream");
    let mut proxy_reader = BufReader::new(&proxy);
    let target = &mut dest.try_clone().expect("Cannot clone stream");

    let mut buffer = [0; 1024];
    loop {
        let nbytes = try_or_skip!(proxy_reader.read(&mut buffer));
        if nbytes == 0 {
            break;
        }
        println!("{} => {:?}", log, &buffer[..nbytes]); // log transferred message
        _ = target.write(&buffer[..nbytes]);
    }
}
