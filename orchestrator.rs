use std::thread;
//use std::collections::HashMap;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use std::time;

const WALLET_MS_IP: [u8; 4] = [127u8, 0u8, 0u8, 1u8];
const WALLET_MS_PORT: u16 = 3333u16;
const ORDER_MS_IP: [u8; 4] = [127u8, 0u8, 0u8, 1u8];
const ORDER_MS_PORT: u16 = 3334u16;

fn main() {
    /*
    let mut threads = Vec::new();
    Arc::new((Mutex::new(String::new()), Condvar::new()));
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // TODO: Legg til kø
        //let transaction_queue = HashMap::new();
        threads.push(
            thread::Builder::new()
                .name("coordinator".to_string())
                .spawn(move || {
                    let mut tries = 1;
                    while !handle_request(&stream) && tries < 5 {
                        tries += 1;
                    }
                }),
        );
    }
    */
    for i in 0..100{
        let mut thread = thread::Builder::new()
        .name("coordinator".to_string())
        .spawn(move || {
            let mut tries = 1;
            while !handle_request() && tries < 5 {
                tries += 1;
                let ten_millis = time::Duration::from_millis(500);
                let now = time::Instant::now();
            
                thread::sleep(ten_millis);
            }
            println!("tries: {}", tries)
        });
        let ten_millis = time::Duration::from_millis(10);
        let now = time::Instant::now();
    
        thread::sleep(ten_millis);
    }
    let ten_millis = time::Duration::from_millis(2000);
    let now = time::Instant::now();

    thread::sleep(ten_millis);

    assert!(now.elapsed() >= ten_millis);
    /*for i in 0..2{
        thread = thread::Builder::new()
            .name("coordinator".to_string())
            .spawn(move || {
                let mut tries = 1;
                while !handle_request() && tries < 5 {
                    tries += 1;
                }
            });
    }*/
    //thread.unwrap().join();
}

fn handle_request() -> bool {
    let mut failed = false;
    // TCP connection duration before timeout
    let timeout = Duration::from_millis(5000);

    // Establish connection to micro services
    let wallet_socket = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(
            WALLET_MS_IP[0],
            WALLET_MS_IP[1],
            WALLET_MS_IP[2],
            WALLET_MS_IP[3],
        )),
        WALLET_MS_PORT,
    );
    let order_socket = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(
            ORDER_MS_IP[0],
            ORDER_MS_IP[1],
            ORDER_MS_IP[2],
            ORDER_MS_IP[3],
        )),
        ORDER_MS_PORT,
    );
    let mut wallet_stream = match TcpStream::connect_timeout(&wallet_socket, timeout) {
        Ok(stream) => stream,
        Err(e) => {
            println!("Failed to create connection to wallet micro service: {}", e);
            return false;
        }
    };
    let mut order_stream = match TcpStream::connect_timeout(&order_socket, timeout) {
        Ok(stream) => stream,
        Err(e) => {
            println!("Failed to create connection to order micro service: {}", e);
            return false;
        }
    };

    // Wallet micro service preperation
    let account = 1u32;
    let amount = 100u32;
    match wallet_stream.write(&account.to_be_bytes()) {
        Ok(_result) => {}
        Err(e) => {
            println!("Failed to write account id to wallet micro service: {}", e);
            failed = true;
        }
    };
    match wallet_stream.write(&amount.to_be_bytes()) {
        Ok(_result) => {}
        Err(e) => {
            println!(
                "Failed to write balance change amount to wallet micro service: {}",
                e
            );
            failed = true;
        }
    };

    // Order micro service preperation
    let user_id = 1u32;
    let amount_of_items = 5u32;
    //let items = [1u32, 2u32, 3u32, 4u32, 5u32];

    match order_stream.write(&user_id.to_be_bytes()) {
        Ok(_result) => {}
        Err(e) => {
            println!("Failed to write user id to order micro service: {}", e);
            failed = true;
        }
    };
    match order_stream.write(&amount_of_items.to_be_bytes()) {
        Ok(_result) => {}
        Err(e) => {
            println!(
                "Failed to write amount of items to order micro service: {}",
                e
            );
            failed = true;
        }
    };
   /* for i in 0..amount_of_items {
        match order_stream.write(&items[i as usize].to_be_bytes()) {
            Ok(_result) => {}
            Err(e) => {
                println!("Failed to write item to order microservice: {}", e);
                failed = true;
            }
        };
    }
    match order_stream.write(&[space]) {
        Ok(_result) => {}
        Err(e) => {
            println!("Failed to write space char to order micro service: {}", e);
            failed = true;
        }
    };
    */

    // Read response
    let mut wallet_response = [0u8];
    let mut order_response = [0u8];

    match wallet_stream.read(&mut wallet_response) {
        Ok(_result) => {}
        Err(e) => {
            println!(
                "Failed to read wallet microservice \"ready to commit\" message: {}",
                e
            );
            failed = true;
        }
    };
    match order_stream.read(&mut order_response) {
        Ok(_result) => {}
        Err(e) => {
            println!(
                "Failed to read order microservice \"ready to commit\" message: {}",
                e
            );
            failed = true;
        }
    };
    println!("wallet response: {}", wallet_response[0]);
    println!("order response: {}", order_response[0]);

    if failed {
        rollback(order_stream, wallet_stream);
        return false;
    }

    if order_response[0] == 1 && wallet_response[0] == 1 {
        println!("Commiting changes");
        let commit_message = 1u32;
        match wallet_stream.write(&commit_message.to_be_bytes()) {
            Ok(_result) => {}
            Err(e) => {
                println!("Wallet service failed to commit: {}", e);
                failed = true;
            }
        };
        match order_stream.write(&commit_message.to_be_bytes()) {
            Ok(_result) => {}
            Err(e) => {
                println!("Order service failed to commit: {}", e);
                failed = true;
            }
        };
        if failed {
            rollback(order_stream, wallet_stream);
            return false;
        }
        return true;
    } else {
        rollback(order_stream, wallet_stream);
        return false;
    }
}

fn rollback(mut order_stream: TcpStream, mut wallet_stream: TcpStream) {
    let mut fails = 0;
    println!("Rolling back transactions");
    let mut order_rolledback = false;
    let mut wallet_rolledback = false;

    while fails < 5 {
        let rollback_message = 2u32;
        match wallet_stream.write(&rollback_message.to_be_bytes()) {
            Ok(_result) => {}
            Err(e) => {
                println!("Wallet microservice rollback write failed: {}", e);
                fails += 1;
            }
        };
        match order_stream.write(&rollback_message.to_be_bytes()) {
            Ok(_result) => {}
            Err(e) => {
                println!("Order microservice rollback write failed: {}", e);
                fails += 1;
            }
        };

        if !wallet_rolledback {
            match wallet_stream.write(&rollback_message.to_be_bytes()) {
                Ok(_result) => { wallet_rolledback = true; }
                Err(e) => {
                    println!("Wallet microservice rollback write failed: {}", e);
                    fails += 1;
                }
            };
        }
        if !order_rolledback {
            match order_stream.write(&rollback_message.to_be_bytes()) {
                Ok(_result) => { order_rolledback = true; }
                Err(e) => {
                    println!("Order microservice rollback write failed: {}", e);
                    fails += 1;
                }
            };
        }

        if order_rolledback && wallet_rolledback {
            return;
        }
    }
    println!("NB: Rollback Failed!");
}
