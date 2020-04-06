use std::thread;
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::net::{TcpStream, TcpListener, IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use std::io::prelude::*;

const wallet_ms_ip: [u8; 4] = [127u8, 0u8, 0u8, 1u8];
const wallet_ms_port: u16 = 3333u16;
const order_ms_ip: [u8; 4] = [127u8, 0u8, 0u8, 1u8];
const order_ms_port: u16 = 3334u16;

fn main() {
    let mut threads = Vec::new();
    Arc::new((Mutex::new(String::new()), Condvar::new()));
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // TODO: Legg til kø
        //let transaction_queue = HashMap::new();
        threads.push(thread::Builder::new().name("coordinator".to_string()).spawn(
            move || {
            handle_request(stream);
        }));
    }
}

fn handle_request(mut client_stream: TcpStream) {
    // TCP connection duration before timeout
    let timeout = Duration::from_millis(5000);

    // Wallet micro service preperation
    let account = 1u32;
    let amount = 100i32;
    let wallet_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(wallet_ms_ip[0], wallet_ms_ip[1], wallet_ms_ip[2], wallet_ms_ip[3])), wallet_ms_port);
    let mut wallet_stream = match TcpStream::connect_timeout(&wallet_socket, timeout) {
        Ok(stream) => stream,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };
    wallet_stream.write(&account.to_be_bytes());
    wallet_stream.write(&amount.to_be_bytes());

     // Order micro service preperation
    let amount_of_items = 5u32;
    let items = [1u32, 2u32, 3u32, 4u32, 5u32];

    let order_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(order_ms_ip[0], order_ms_ip[1], order_ms_ip[2], order_ms_ip[3])), order_ms_port);
    let mut order_stream = match TcpStream::connect_timeout(&order_socket, timeout) {
        Ok(stream) => stream,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };
    order_stream.write(&amount_of_items.to_be_bytes());
    for i in 0..amount_of_items {
        order_stream.write(&items[i as usize].to_be_bytes());
    }

    // Read response
    let mut wallet_response = [0u8];
    let mut order_response = [0u8];
    wallet_stream.read(&mut wallet_response);
    order_stream.read(&mut order_response);

    if order_response[0] == 1 && wallet_response[0] == 1 {
        // Both services ready, go ahead and commit
        let commit_message = [1u8];
        wallet_stream.write(&commit_message);
        order_stream.write(&commit_message);
    }
    else {
        // Error, rollback
        let rollback_message = [2u8];
        wallet_stream.write(&rollback_message);
        order_stream.write(&rollback_message);
    }
}