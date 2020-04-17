use std::thread;
//use std::collections::HashMap;
use std::io::prelude::*;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use std::time;
use std::fs;
extern crate serde;
extern crate serde_json;
use serde::{Serialize, Deserialize};
use serde_json::Result;

//const WALLET_MS_IP: [u8; 4] = [10u8, 128u8, 0u8, 9u8]; //35.202.15.128
const WALLET_MS_PORT: u16 = 3332u16;
//const ORDER_MS_IP: [u8; 4] = [10u8, 128u8, 0u8, 10u8]; //34.67.42.245
const ORDER_MS_PORT: u16 = 3335u16;

fn main() {
    let contents = fs::read_to_string("./addresses")
    .expect("Something went wrong reading the file");
    println!("{}",contents);
    let addresses:Vec<&str> = contents.split(" ").collect();
    let listen: &str = addresses[0];
    let walletnumbers:Vec<&str> = addresses[1].split(".").collect();
    let ordernumbers:Vec<&str> = addresses[2].split(".").collect();
    //println!("{}", numbers[0].as_bytes());
    println!("{}", ordernumbers[0]);
    let wallet_ip: [u8; 4] = [walletnumbers[0].parse::<u8>().unwrap(), walletnumbers[1].parse::<u8>().unwrap(), walletnumbers[2].parse::<u8>().unwrap(), walletnumbers[3].parse::<u8>().unwrap()];
    let order_ip: [u8; 4] = [ordernumbers[0].parse::<u8>().unwrap(), ordernumbers[1].parse::<u8>().unwrap(), ordernumbers[2].parse::<u8>().unwrap(), ordernumbers[3].parse::<u8>().unwrap()];
    println!("{:?}", wallet_ip);
    println!("{:?}", order_ip);

    
    let mut threads = Vec::new();
    Arc::new((Mutex::new(String::new()), Condvar::new()));
    let listener = TcpListener::bind(listen.to_owned()+":3000").unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        {
        threads.push(
            thread::Builder::new()
                .name("coordinator".to_string())
                .spawn(move || {
                    let (status, account, amount, user_id, amount_of_items, items) = read_http_request(&stream);
                    if status == 2 {
                        let response = "HTTP/1.1 400 JSON could not be serialized, check syntax\n\n";
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                    else if status == 3 {
                        let response = "HTTP/1.1 404 Endpoint not found\n\n";
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                    else if status == 4 {
                        let response = "HTTP/1.1 404 Only POST is supported on this endpoint\n\n";
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                    else if status == 5 {
                        let response = "HTTP/1.1 400 Amout of items dose not match number of entries in items array\n\n";
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                    else {
                        let mut tries = 0;
                        while tries < 5 {
                            if handle_request(&wallet_ip, &order_ip, account, amount, user_id, amount_of_items, &items, &stream) {
                                break;
                            }
                            else {
                                tries += 1;
                                println!("Failed attempt #{}", tries);
                            }
                        }
                        if tries >= 5 {
                            let response = "HTTP/1.1 500\n\nCould not fulfill order";
                            stream.write_all(response.as_bytes()).unwrap();
                            println!("Could not fulfilll order");
                        }
                        else {
                            let q = '"'.escape_default();
                            //let data = r#"{"output": "success"}"#;
                            //let response = "HTTP/1.1 200 OK\n\n".to_owned() + data;
                            let response = "HTTP/1.1 200 OK\n\nsuccess";
                            stream.write_all(response.as_bytes()).unwrap();
                            println!("Order fulfilled");
                        }
                    }
                }),
        );
        }
    }
    
    /*for i in 0..10{
        let mut thread = thread::Builder::new()
        .name("coordinator".to_string())
        .spawn(move || {
            let mut tries = 1;
            while !handle_request(&wallet_ip, &order_ip) && tries < 5 {
                tries += 1;
                let ten_millis = time::Duration::from_millis(500);
                let now = time::Instant::now();
            
                thread::sleep(ten_millis);
            }
            println!("tries: {}", tries)
        });
    } */
    /*for i in 0..2{
        thread = thread::Builder::new()
            .name("coordinator".to_string())
            .spawn(move || {
                let mut tries = 1;
                while !handle_request() && tries < 5 {
                    tries += 1;
                }
            });
    }

    thread.unwrap().join();
    */
}

fn handle_request(wallet_ip: &[u8; 4], order_ip: &[u8; 4], account: u32, amount:u32, user_id: u32, amount_of_items: u32, items: &Vec<u32>, mut client_stream: &TcpStream) -> bool {
    /*
    let response = "HTTP/1.1 200 OK\n\n<html><body>Message Recieved</body></html>";
    client_stream.write_all(response.as_bytes()).unwrap();
    */
    /*
    // Wallet micro service preperation
    let account = 1u32;
    let amount = 100u32;
    // Order micro service preperation
    let user_id = 1u32;
    let amount_of_items = 5u32;
    let items = [1u32, 2u32, 3u32, 4u32, 5u32];
    */

    let mut failed = false;
    // TCP connection duration before timeout
    let timeout = Duration::from_millis(5000);

    // Establish connection to micro services
    let wallet_socket = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(
            wallet_ip[0],
            wallet_ip[1],
            wallet_ip[2],
            wallet_ip[3],
        )),
        WALLET_MS_PORT,
    );
    let order_socket = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(
            order_ip[0],
            order_ip[1],
            order_ip[2],
            order_ip[3],
        )),
        ORDER_MS_PORT,
    );
    let mut order_stream = match TcpStream::connect_timeout(&order_socket, timeout) {
        Ok(stream) => stream,
        Err(e) => {
            println!("Failed to create connection to order micro service: {}", e);
            return false;
        }
    };

    let mut wallet_stream = match TcpStream::connect_timeout(&wallet_socket, timeout) {
        Ok(stream) => stream,
        Err(e) => {
            println!("Failed to create connection to wallet micro service: {}", e);
            return false;
        }
    };

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
    
    for i in 0..amount_of_items {
        match order_stream.write(&items[i as usize].to_be_bytes()) {
            Ok(_result) => {}
            Err(e) => {
                println!("Failed to write item to order microservice: {}", e);
                failed = true;
            }
        };
    }
    
    /*match order_stream.write(&[space]) {
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
    let response_definitions = ["Error reading data from orchestrator", "OK Prepare", "OK Commit", "User has uncommited transactions", "Could not connect to database", "Could not start transaction", "Error with transaction query", "Transaction rolled back", "Transaction never started", "Error querying from wallet table", "Wrong format on result from wallet table", "User does not exist", "Balance too low"];
    print!("wallet response: {}", wallet_response[0]);
    if wallet_response[0] < 14 {
        println!(" ({})", response_definitions[wallet_response[0] as usize]);
    }
    else {
        println!();
    }
    print!("order response: {}", order_response[0]);
    if order_response[0] < 9 {
        print!(" ({})", response_definitions[order_response[0] as usize]);
    }
    else {
        println!();
    }

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

fn read_http_request(client_stream: &TcpStream) -> (u8, u32, u32, u32, u32, Vec<u32>){
    let mut reader = BufReader::new(client_stream);

    // Første linje i header
    let mut http_request_definition = String::new();
    let _result = reader.by_ref().read_line(&mut http_request_definition);
    let http_request_definition_split: Vec<&str> = http_request_definition.split_whitespace().collect();
    println!("{}", http_request_definition);

    // Alle headers
    let mut http_request_headers = Vec::new();
    http_request_headers.push(http_request_definition.clone());

    let mut has_body = false;
    if http_request_definition_split[0] == "POST" {
        has_body = true;
    }
    let mut body: Vec<u8>  = vec![];
    for line in reader.by_ref().lines() {
        let line_uw = line.unwrap();
        println!("{}", line_uw);
        if line_uw.len() > 15 {
            if &line_uw[..15] == "Content-Length:"{
                body = vec![0;(&line_uw[16..]).parse().unwrap()]
            }
        }
        if line_uw == "" { 
            if has_body {
                let _result = reader.by_ref().read_exact(&mut body);
            }
            break;
        }
        http_request_headers.push(line_uw);
    }
    if !has_body {
        return (4, 0, 0, 0, 0, vec![0]);
    }
    if http_request_definition_split[1] == "/purchase" {
        let mut body_string = String::new();
        for byte in body {
            body_string.push(byte as char);
        }
        println!("body string: {}", body_string);
        let order: Order = match serde_json::from_str(&body_string[..]) {
            Ok(data) => data,
            Err(e) => {
                println!("JSON serilization failed: {}", e);
                return (2, 0,0,0,0, vec![0]);
            }
        };
        if order.amount_of_items != order.items.len() as u32 {
            println!("Amount of items dose not match item array length");
            return (5, 0, 0, 0, 0, vec![0]);
        }
        println!("JSON read succesfull");
        return(1, order.account, order.amount, order.user_id, order.amount_of_items, order.items);
    }
    else {
        return (3, 0, 0, 0, 0, vec![0]);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Order {
    account: u32,
    amount: u32,
    user_id: u32,
    amount_of_items: u32,
    items: Vec<u32>
}
