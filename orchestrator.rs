use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

// Serde is for json support
extern crate serde;
extern crate serde_json;
use serde::{Deserialize, Serialize};

const WALLET_MS_PORT: u16 = 3332u16;
const ORDER_MS_PORT: u16 = 3335u16;

fn main() {
    // IP addresses are of micro services are read from file
    let contents =
        fs::read_to_string("./addresses").expect("Something went wrong reading the file");
    let addresses: Vec<&str> = contents.split(" ").collect();
    let listen: &str = addresses[0];
    println!("Orchestrator running on {}:3000", listen);
    let walletnumbers: Vec<&str> = addresses[1].split(".").collect();
    let ordernumbers: Vec<&str> = addresses[2].split(".").collect();
    let wallet_ip: [u8; 4] = [
        walletnumbers[0].parse::<u8>().unwrap(),
        walletnumbers[1].parse::<u8>().unwrap(),
        walletnumbers[2].parse::<u8>().unwrap(),
        walletnumbers[3].parse::<u8>().unwrap(),
    ];
    let order_ip: [u8; 4] = [
        ordernumbers[0].parse::<u8>().unwrap(),
        ordernumbers[1].parse::<u8>().unwrap(),
        ordernumbers[2].parse::<u8>().unwrap(),
        ordernumbers[3].parse::<u8>().unwrap(),
    ];

    // The server listens for http requests on port 3000
    let mut threads = Vec::new();
    let listener = TcpListener::bind(listen.to_owned() + ":3000").unwrap();
    for stream in listener.incoming() {
        // Http requests are handled by individual threads
        let mut stream = stream.unwrap();
        {
            threads.push(
                thread::Builder::new()
                    .name("coordinator".to_string())
                    .spawn(move || {
                        // Reads the http request
                        let (status, account, amount, user_id, items) = read_http_request(&stream);
                        if status == 2 {
                            // Json could not be parsed
                            let response =
                                "HTTP/1.1 400 JSON could not be serialized, check syntax\n\n";
                            stream.write_all(response.as_bytes()).unwrap();
                        } else if status == 3 {
                            // Request wasn't sent to recognized endpoint
                            let response = "HTTP/1.1 404 Endpoint not found\n\n";
                            stream.write_all(response.as_bytes()).unwrap();
                        } else if status == 4 {
                            send_file(stream, "client/index.html");
                        }
                        else if status == 5 {
                            send_file(stream, "client/rust-logo.png");
                        } 
                        else if status == 1 {
                            // If the transaction fails the orchestrator will retry up to 5 times
                            let mut tries = 0;
                            let mut status_code = 0;
                            while tries < 5 {
                                status_code = handle_request(
                                    &wallet_ip, &order_ip, account, amount, user_id, &items,
                                );
                                if status_code == 1 {
                                    break;
                                } else {
                                    tries += 1;
                                    println!("Failed attempt #{}", tries);
                                }
                            }
                            if tries >= 5 {
                                // After 5 fails we accept defeat and return an error message
                                let response_definitions = [
                                    "Error reading data from orchestrator",
                                    "OK Prepare",
                                    "OK Commit",
                                    "User has uncommited transactions",
                                    "Could not connect to database",
                                    "Could not start transaction",
                                    "Error with transaction query",
                                    "Transaction rolled back",
                                    "Transaction never started",
                                    "Error querying from wallet table",
                                    "Wrong format on result from wallet table",
                                    "User does not exist",
                                    "Balance too low",
                                    "Not in stock"
                                ];
                                let mut response = String::new();
                                if status_code > 6 && status_code < 21 {
                                    response.push_str("HTTP/1.1 406\n\n");
                                    response.push_str(response_definitions[(status_code-7) as usize]);
                                }
                                else if status_code == 1 {
                                    response.push_str("HTTP/1.1 200 OK\n\nsuccess");
                                }
                                else if status_code == 2 {
                                    response.push_str("HTTP/1.1 500\n\nFailed to create connection to order micro service");
                                }
                                else if status_code == 3 {
                                    response.push_str("HTTP/1.1 500\n\nFailed to create connection to wallet micro service");
                                }
                                else if status_code == 4 {
                                    response.push_str("HTTP/1.1 500\n\nA TCP connection failed unexpectedly");
                                }
                                else if status_code == 5 {
                                    response.push_str("HTTP/1.1 500\n\nWallet service failed to commit twice");
                                }
                                else if status_code == 6 {
                                    response.push_str("HTTP/1.1 500\n\nOrder service failed to commit twice");
                                }
                                else {
                                    response.push_str("HTTP/1.1 500\n\nUnkown failure");
                                }
                                
                                stream.write_all(response.as_bytes()).unwrap();
                                println!("Could not fulfilll order");
                            } else {
                                let response = "HTTP/1.1 200 OK\n\nsuccess";
                                stream.write_all(response.as_bytes()).unwrap();
                                println!("Order fulfilled");
                            }
                        }
                    }),
            );
        }
    }
}

fn send_file(mut stream: TcpStream, file_name: &str){
    let mut file_bytes_vec: Vec<u8> = Vec::new();
    let response = "HTTP/1.1 200 OK\n\n";
    for byte in response.as_bytes() {
        file_bytes_vec.push(*byte);
    }

    let mut file = match fs::File::open(file_name) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to read file: {}", e);
            return;
        }
    };
    let _result = file.read_to_end(&mut file_bytes_vec);
    let file_bytes: &[u8] = &file_bytes_vec;
    match stream.write_all(file_bytes) {
        Ok(_result) => {},
        Err(e) => {
            println!("Failed to write file to TCP stream: {}", e);
            return;
        }
    };
}

fn handle_request(
    wallet_ip: &[u8; 4],
    order_ip: &[u8; 4],
    account: u32,
    amount: u32,
    user_id: u32,
    items: &Vec<u32>,
) -> u8 {
    let mut failed = false;
    // TCP connection duration before timeout
    let timeout = Duration::from_millis(5000);

    // Create sockets for micro services
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

    // Establish TCP connection to micro services using our sockets
    let mut order_stream = match TcpStream::connect_timeout(&order_socket, timeout) {
        Ok(stream) => stream,
        Err(e) => {
            println!("Failed to create connection to order micro service: {}", e);
            return 2;
        }
    };

    let mut wallet_stream = match TcpStream::connect_timeout(&wallet_socket, timeout) {
        Ok(stream) => stream,
        Err(e) => {
            println!("Failed to create connection to wallet micro service: {}", e);
            return 3;
        }
    };

    // Write account number and charge amount to wallet microservice
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

    // Write user id, amount of items and items to order microservice
    match order_stream.write(&user_id.to_be_bytes()) {
        Ok(_result) => {}
        Err(e) => {
            println!("Failed to write user id to order micro service: {}", e);
            failed = true;
        }
    };
    println!("AMOUNT: {}", &items.len());
    let amountofitems = items.len() as u32;

    match order_stream.write(&amountofitems.to_be_bytes()) {
        Ok(_result) => {}
        Err(e) => {
            println!(
                "Failed to write amount of items to order micro service: {}",
                e
            );
            failed = true;
        }
    };

    for item in items {
        match order_stream.write(&item.to_be_bytes()) {
            Ok(_result) => {}
            Err(e) => {
                println!("Failed to write item to order microservice: {}", e);
                failed = true;
            }
        };
    }

    let mut wallet_response = [0u8];
    let mut order_response = [0u8];
    // Read response from microservices
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

    // Check if microservices succeded, print cause of error if a microservice fails
    let response_definitions = [
        "Error reading data from orchestrator",
        "OK Prepare",
        "OK Commit",
        "User has uncommited transactions",
        "Could not connect to database",
        "Could not start transaction",
        "Error with transaction query",
        "Transaction rolled back",
        "Transaction never started",
        "Error querying from wallet table",
        "Wrong format on result from wallet table",
        "User does not exist",
        "Balance too low",
        "Not in stock"
    ];
    print!("wallet response: {}", wallet_response[0]);
    if wallet_response[0] < 14 {
        println!(" ({})", response_definitions[wallet_response[0] as usize]);
    } else {
        println!();
    }
    print!("order response: {}", order_response[0]);
    if order_response[0] < 14 {
        print!(" ({})", response_definitions[order_response[0] as usize]);
    } else {
        println!();
    }

    // If any of the TCP read/writes failed it will roll back
    if failed {
        rollback(order_stream, wallet_stream);
        return 4;
    }

    // The microservices respond with 1 if they are ready to commit
    if order_response[0] == 1 && wallet_response[0] == 1 {
        println!("Commiting changes");
        let commit_message = 1u32;
        let mut wallet_commit_failed = false;
        let mut order_commit_failed = false;
        // Tell microservices to commit.
        match wallet_stream.write(&commit_message.to_be_bytes()) {
            Ok(_result) => {}
            Err(e) => {
                println!("NB! Wallet service failed to commit: {}", e);
                wallet_commit_failed = true;
            }
        };
        // If sending commit to one of the services fails it will retry once.
        if wallet_commit_failed {
            match wallet_stream.write(&commit_message.to_be_bytes()) {
                Ok(_result) => {}
                Err(e) => {
                    println!("NB! Wallet service failed to commit twice. Contact system administrator. Error: {}", e);
                    return 5;
                }
            };
        }
        match order_stream.write(&commit_message.to_be_bytes()) {
            Ok(_result) => {}
            Err(e) => {
                println!("NB! Order service failed to commit: {}", e);
                order_commit_failed = true;
            }
        };
        if order_commit_failed {
            match order_stream.write(&commit_message.to_be_bytes()) {
                Ok(_result) => {}
                Err(e) => {
                    println!("NB! Order service failed to commit twice. Contact system administrator. Error: {}", e);
                    return 6;
                }
            };
        }
        return 1;
    } else {
        // If one of the microservices failed to prepare the orchestrator will rollback both and retry
        rollback(order_stream, wallet_stream);
        if wallet_response[0] != 1 {
            return 7 + wallet_response[0];
        }
        else {
            return 7 + order_response[0];
        }
    }
}

fn rollback(mut order_stream: TcpStream, mut wallet_stream: TcpStream) {
    let mut fails = 0;
    println!("Rolling back transactions");
    let mut order_rolledback = false;
    let mut wallet_rolledback = false;
    // The message 2 tells micro services to rollback
    let rollback_message = 2u32;
    match wallet_stream.write(&rollback_message.to_be_bytes()) {
        Ok(_result) => wallet_rolledback = true,
        Err(e) => {
            println!("Wallet microservice rollback write failed: {}", e);
            fails += 1;
        }
    };
    match order_stream.write(&rollback_message.to_be_bytes()) {
        Ok(_result) => order_rolledback = true,
        Err(e) => {
            println!("Order microservice rollback write failed: {}", e);
            fails += 1;
        }
    };
    // Orchestrator tries up to 5 times combined to send rollback messages if they fail
    while fails < 5 {
        if !wallet_rolledback {
            match wallet_stream.write(&rollback_message.to_be_bytes()) {
                Ok(_result) => {
                    wallet_rolledback = true;
                }
                Err(e) => {
                    println!("Wallet microservice rollback write failed: {}", e);
                    fails += 1;
                }
            };
        }
        if !order_rolledback {
            match order_stream.write(&rollback_message.to_be_bytes()) {
                Ok(_result) => {
                    order_rolledback = true;
                }
                Err(e) => {
                    println!("Order microservice rollback write failed: {}", e);
                    fails += 1;
                }
            };
        }
        if order_rolledback && wallet_rolledback {
            println!("Rollback Succesfull");
            return;
        }
    }
    println!("NB: Rollback Failed!");
}

fn read_http_request(client_stream: &TcpStream) -> (u8, u32, u32, u32, Vec<u32>) {
    let mut reader = BufReader::new(client_stream);

    // Reads the first line in header in the HTTP request
    // Expected to be "POST /purchase HTTP/1.1" in this case
    let mut http_request_definition = String::new();
    let _result = reader.by_ref().read_line(&mut http_request_definition);
    let http_request_definition_split: Vec<&str> =
        http_request_definition.split_whitespace().collect();
    println!("{}", http_request_definition);

    // Reads the rest of the headers
    let mut http_request_headers = Vec::new();
    http_request_headers.push(http_request_definition.clone());

    // If it is a post request we know to read beyond the headers
    let mut has_body = false;
    if http_request_definition_split[0] == "POST" {
        has_body = true;
    }
    let mut body: Vec<u8> = vec![];
    for line in reader.by_ref().lines() {
        let line_uw = line.unwrap();
        println!("{}", line_uw);
        // We look for the content length header so we know how far to read
        if line_uw.len() > 15 {
            if &String::from(&line_uw).to_lowercase()[..15] == "content-length:" {
                body = vec![0; (&line_uw[16..]).parse().unwrap()]
            }
        }
        // If we encounter an empty line it means that the headers are finished
        // If the requset has a body we read more
        if line_uw == "" {
            if has_body {
                let _result = reader.by_ref().read_exact(&mut body);
            }
            break;
        }
        http_request_headers.push(line_uw);
    }
    // If the request is GET / we return the index.html client
    if http_request_definition_split[0] == "GET" {
        if http_request_definition_split[1] == "/" {
            return (4, 0, 0, 0, vec![0]);
        }
        else if http_request_definition_split[1] == "/favicon.ico" {
            return (5, 0, 0, 0, vec![0]);
        }
    }
    // If the request isn't a post request we send an error message
    if !has_body {
        return (3, 0, 0, 0, vec![0]);
    }
    // We check that the endpoint is correct
    if http_request_definition_split[1] == "/purchase" {
        // We read the body as text
        let mut body_string = String::new();
        for byte in body {
            body_string.push(byte as char);
        }
        println!("body: {}", body_string);
        // We parse the text as a JSON object
        let order: Order = match serde_json::from_str(&body_string[..]) {
            Ok(data) => data,
            Err(e) => {
                println!("JSON serilization failed: {}", e);
                return (2, 0, 0, 0, vec![0]);
            }
        };
        println!("JSON read succesfull");
        return (1, order.account, order.amount, order.user_id, order.items);
    } else {
        return (3, 0, 0, 0, vec![0]);
    }
}

// We define the expected contents of the json in the POST request
#[derive(Serialize, Deserialize, Debug)]
struct Order {
    account: u32,
    amount: u32,
    user_id: u32,
    items: Vec<u32>,
}
