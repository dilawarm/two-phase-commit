use std::thread;
//use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::net::{TcpStream, TcpListener, IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use std::io::prelude::*;

const WALLET_MS_IP: [u8; 4] = [127u8, 0u8, 0u8, 1u8];
const WALLET_MS_PORT: u16 = 3333u16;
const ORDER_MS_IP: [u8; 4] = [127u8, 0u8, 0u8, 1u8];
const ORDER_MS_PORT: u16 = 3334u16;

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
                let mut tries = 1;
                while !handle_request(&stream) && tries < 5 {
                    tries += 1;
                }
        }));
    }
}

fn handle_request(mut _client_stream: &TcpStream) -> bool {
    // TCP connection duration before timeout
    let timeout = Duration::from_millis(5000);

    // Wallet micro service preperation
    let account = 1u32;
    let amount = 100i32;
    let wallet_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(WALLET_MS_IP[0], WALLET_MS_IP[1], WALLET_MS_IP[2], WALLET_MS_IP[3])), WALLET_MS_PORT);
    let mut wallet_stream = match TcpStream::connect_timeout(&wallet_socket, timeout) {
        Ok(stream) => stream,
        Err(e) => {
            println!("Error: {}", e);
            return false;
        }
    };
    match wallet_stream.write(&account.to_be_bytes()){
        Ok(_result) => {},
        Err(e) => {
            println!("Error: {}", e);
            return false;
        }
    };
    match wallet_stream.write(&amount.to_be_bytes()){
        Ok(_result) => {},
        Err(e) => {
            println!("Error: {}", e);
            return false;
        }
    };

    // Order micro service preperation
    let user_id = 1u32;
    let amount_of_items = 5u32;
    let items = [1u32, 2u32, 3u32, 4u32, 5u32];

    let order_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(ORDER_MS_IP[0], ORDER_MS_IP[1], ORDER_MS_IP[2], ORDER_MS_IP[3])), ORDER_MS_PORT);
    let mut order_stream = match TcpStream::connect_timeout(&order_socket, timeout) {
        Ok(stream) => stream,
        Err(e) => {
            println!("Error: {}", e);
            return false;
        }
    };
    match order_stream.write(&user_id.to_be_bytes()){
        Ok(_result) => {},
        Err(e) => {
            println!("Error: {}", e);
            return false;
        }
    };
    match order_stream.write(&amount_of_items.to_be_bytes()){
        Ok(_result) => {},
        Err(e) => {
            println!("Error: {}", e);
            return false;
        }
    };
    for i in 0..amount_of_items {
        match order_stream.write(&items[i as usize].to_be_bytes()){
            Ok(_result) => {},
            Err(e) => {
                println!("Error: {}", e);
                return false;
            }
        };
    }

    // Read response
    let mut wallet_response = [0u8];
    let mut order_response = [0u8];
    match wallet_stream.read(&mut wallet_response){
        Ok(_result) => {},
        Err(e) => {
            println!("Error: {}", e);
            return false;
        }
    };
    match order_stream.read(&mut order_response){
        Ok(_result) => {},
        Err(e) => {
            println!("Error: {}", e);
            return false;
        }
    };

    if order_response[0] == 1 && wallet_response[0] == 1 {
        // Both services ready, go ahead and commit
        let commit_message = [1u8];
        match wallet_stream.write(&commit_message){
            Ok(_result) => {},
            Err(e) => {
                println!("Error: {}", e);
                return false;
            }
        };
        match order_stream.write(&commit_message){
            Ok(_result) => {},
            Err(e) => {
                println!("Error: {}", e);
                return false;
            }
        };
        return true;
    }
    else {
        // Error, rollback
        let rollback_message = [2u8];
        match wallet_stream.write(&rollback_message){
            Ok(_result) => {},
            Err(e) => {
                println!("Error: {}", e);
                return false;
            }
        };
        match order_stream.write(&rollback_message){
            Ok(_result) => {},
            Err(e) => {
                println!("Error: {}", e);
                return false;
            }
        };
        return false;
    }
}