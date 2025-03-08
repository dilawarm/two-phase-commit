//! Orchestrator Service for E-commerce System
//!
//! This module implements a transaction orchestrator that coordinates between
//! wallet and order microservices using a two-phase commit protocol.
//! 
//! The orchestrator:
//! 1. Receives purchase requests from clients via HTTP
//! 2. Coordinates the transaction between wallet and order services
//! 3. Ensures atomicity by implementing two-phase commit (prepare/commit/rollback)
//! 4. Handles error cases and service failures gracefully
//! 
//! The system uses TCP for inter-service communication and implements
//! retry mechanisms for improved reliability.


use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

// Serde for JSON serialization/deserialization
use serde::{Deserialize, Serialize};

// Constants
const WALLET_MS_PORT: u16 = 3332;
const ORDER_MS_PORT: u16 = 3335;
const HTTP_OK: &str = "HTTP/1.1 200 OK\n\n";
const HTTP_BAD_REQUEST: &str = "HTTP/1.1 400 JSON could not be serialized, check syntax\n\n";
const HTTP_NOT_FOUND: &str = "HTTP/1.1 404 Endpoint not found\n\n";
const HTTP_NOT_ACCEPTABLE: &str = "HTTP/1.1 406\n\n";
const HTTP_SERVER_ERROR: &str = "HTTP/1.1 500\n\n";
const MAX_RETRY_ATTEMPTS: usize = 5;
const TCP_TIMEOUT_MS: u64 = 5000;

/// Response codes from microservices
const RESPONSE_DEFINITIONS: [&str; 14] = [
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

/// Represents an order request from the client
#[derive(Serialize, Deserialize, Debug)]
struct Order {
    account: u32,
    amount: u32,
    user_id: u32,
    items: Vec<u32>,
}

/// Represents the result of processing an HTTP request
enum HttpRequestResult {
    Purchase { account: u32, amount: u32, user_id: u32, items: Vec<u32> },
    ServeFile(String),
    BadRequest,
    NotFound,
}

/// Represents the result of a transaction
enum TransactionResult {
    Success,
    OrderServiceConnectionFailed,
    WalletServiceConnectionFailed,
    TcpConnectionFailed,
    WalletServiceCommitFailed,
    OrderServiceCommitFailed,
    MicroserviceError(u8),
}

/// Main function - entry point of the application
fn main() {
    // Read microservice IP addresses from configuration file
    let (listen_addr, wallet_ip, order_ip) = read_configuration();
    println!("Orchestrator running on {}:3000", listen_addr);
    
    // Start the HTTP server
    let listener = TcpListener::bind(format!("{}:3000", listen_addr))
        .expect("Failed to bind to address");
    
    println!("Server listening on port 3000");
    
    // Handle incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Spawn a new thread for each connection
                thread::Builder::new()
                    .name("coordinator".to_string())
                    .spawn(move || handle_connection(stream, &wallet_ip, &order_ip))
                    .expect("Failed to create thread");
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}

/// Reads configuration from the addresses file
fn read_configuration() -> (String, [u8; 4], [u8; 4]) {
    let contents = fs::read_to_string("./addresses")
        .expect("Failed to read addresses file");
    
    let addresses: Vec<&str> = contents.split(' ').collect();
    let listen_addr = addresses[0].to_string();
    
    let wallet_ip = parse_ip_address(addresses[1]);
    let order_ip = parse_ip_address(addresses[2]);
    
    (listen_addr, wallet_ip, order_ip)
}

/// Parses an IP address string into [u8; 4]
fn parse_ip_address(ip_str: &str) -> [u8; 4] {
    let parts: Vec<&str> = ip_str.split('.').collect();
    [
        parts[0].parse().unwrap(),
        parts[1].parse().unwrap(),
        parts[2].parse().unwrap(),
        parts[3].parse().unwrap(),
    ]
}

/// Handles a client connection
fn handle_connection(mut stream: TcpStream, wallet_ip: &[u8; 4], order_ip: &[u8; 4]) {
    // Parse the HTTP request
    let request_result = read_http_request(&stream);
    
    match request_result {
        HttpRequestResult::BadRequest => {
            stream.write_all(HTTP_BAD_REQUEST.as_bytes()).unwrap();
        },
        HttpRequestResult::NotFound => {
            stream.write_all(HTTP_NOT_FOUND.as_bytes()).unwrap();
        },
        HttpRequestResult::ServeFile(filename) => {
            send_file(stream, &filename);
        },
        HttpRequestResult::Purchase { account, amount, user_id, items } => {
            process_purchase(stream, wallet_ip, order_ip, account, amount, user_id, items);
        }
    }
}

/// Processes a purchase request with retry logic
fn process_purchase(
    mut stream: TcpStream, 
    wallet_ip: &[u8; 4], 
    order_ip: &[u8; 4], 
    account: u32, 
    amount: u32, 
    user_id: u32, 
    items: Vec<u32>
) {
    // Try the transaction up to MAX_RETRY_ATTEMPTS times
    for attempt in 1..=MAX_RETRY_ATTEMPTS {
        let result = handle_request(wallet_ip, order_ip, account, amount, user_id, &items);
        
        match result {
            TransactionResult::Success => {
                stream.write_all(format!("{}{}", HTTP_OK, "success").as_bytes()).unwrap();
                println!("Order fulfilled");
                return;
            },
            // Continue retrying for these errors
            TransactionResult::TcpConnectionFailed |
            TransactionResult::OrderServiceConnectionFailed |
            TransactionResult::WalletServiceConnectionFailed |
            TransactionResult::MicroserviceError(_) => {
                println!("Failed attempt #{}", attempt);
                if attempt == MAX_RETRY_ATTEMPTS {
                    send_error_response(&mut stream, &result);
                    println!("Could not fulfill order after {} attempts", MAX_RETRY_ATTEMPTS);
                }
            },
            // Fatal errors - don't retry
            TransactionResult::WalletServiceCommitFailed |
            TransactionResult::OrderServiceCommitFailed => {
                send_error_response(&mut stream, &result);
                println!("Could not fulfill order - fatal error");
                return;
            }
        }
    }
}

/// Sends an appropriate error response based on the transaction result
fn send_error_response(stream: &mut TcpStream, result: &TransactionResult) {
    let error_message = match result {
        TransactionResult::OrderServiceConnectionFailed => 
            format!("{}Failed to create connection to order micro service", HTTP_SERVER_ERROR),
        TransactionResult::WalletServiceConnectionFailed => 
            format!("{}Failed to create connection to wallet micro service", HTTP_SERVER_ERROR),
        TransactionResult::TcpConnectionFailed => 
            format!("{}A TCP connection failed unexpectedly", HTTP_SERVER_ERROR),
        TransactionResult::WalletServiceCommitFailed => 
            format!("{}Wallet service failed to commit twice", HTTP_SERVER_ERROR),
        TransactionResult::OrderServiceCommitFailed => 
            format!("{}Order service failed to commit twice", HTTP_SERVER_ERROR),
        TransactionResult::MicroserviceError(code) if *code > 6 && *code < 21 => 
            format!("{}{}", HTTP_NOT_ACCEPTABLE, RESPONSE_DEFINITIONS[(*code-7) as usize]),
        _ => format!("{}Unknown failure", HTTP_SERVER_ERROR),
    };
    
    stream.write_all(error_message.as_bytes()).unwrap();
}

/// Sends a file to the client
fn send_file(mut stream: TcpStream, file_name: &str) {
    let mut file_bytes_vec: Vec<u8> = Vec::new();
    
    // Add HTTP header
    file_bytes_vec.extend_from_slice(HTTP_OK.as_bytes());
    
    // Read file contents
    match fs::File::open(file_name) {
        Ok(mut file) => {
            if let Err(e) = file.read_to_end(&mut file_bytes_vec) {
                eprintln!("Failed to read file: {}", e);
                return;
            }
        },
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            return;
        }
    };
    
    // Send file to client
    if let Err(e) = stream.write_all(&file_bytes_vec) {
        eprintln!("Failed to write file to TCP stream: {}", e);
    }
}

/// Handles a transaction request using the two-phase commit protocol
fn handle_request(
    wallet_ip: &[u8; 4],
    order_ip: &[u8; 4],
    account: u32,
    amount: u32,
    user_id: u32,
    items: &Vec<u32>,
) -> TransactionResult {
    // Create sockets for microservices
    let wallet_socket = create_socket(wallet_ip, WALLET_MS_PORT);
    let order_socket = create_socket(order_ip, ORDER_MS_PORT);
    
    // Establish TCP connections to microservices
    let timeout = Duration::from_millis(TCP_TIMEOUT_MS);
    
    let order_stream = match TcpStream::connect_timeout(&order_socket, timeout) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to create connection to order micro service: {}", e);
            return TransactionResult::OrderServiceConnectionFailed;
        }
    };

    let wallet_stream = match TcpStream::connect_timeout(&wallet_socket, timeout) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to create connection to wallet micro service: {}", e);
            return TransactionResult::WalletServiceConnectionFailed;
        }
    };
    
    // Phase 1: Prepare
    if let Err(_) = prepare_transaction(wallet_stream, order_stream, account, amount, user_id, items) {
        return TransactionResult::TcpConnectionFailed;
    }
    
    // Get the prepared streams back
    let (mut wallet_stream, mut order_stream, wallet_response, order_response) = 
        match get_prepare_responses(wallet_stream, order_stream) {
            Ok(result) => result,
            Err(_) => return TransactionResult::TcpConnectionFailed,
        };
    
    // Log responses
    log_microservice_responses(wallet_response, order_response);
    
    // Phase 2: Commit or Rollback
    if order_response == 1 && wallet_response == 1 {
        // Both services are ready to commit
        println!("Committing changes");
        
        // Tell wallet service to commit
        if let Err(_) = send_commit_message(&mut wallet_stream) {
            return TransactionResult::WalletServiceCommitFailed;
        }
        
        // Tell order service to commit
        if let Err(_) = send_commit_message(&mut order_stream) {
            return TransactionResult::OrderServiceCommitFailed;
        }
        
        TransactionResult::Success
    } else {
        // At least one service failed to prepare, rollback both
        rollback(order_stream, wallet_stream);
        
        // Return appropriate error code
        if wallet_response != 1 {
            TransactionResult::MicroserviceError(7 + wallet_response)
        } else {
            TransactionResult::MicroserviceError(7 + order_response)
        }
    }
}

/// Creates a socket address from IP and port
fn create_socket(ip: &[u8; 4], port: u16) -> SocketAddr {
    SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])),
        port,
    )
}

/// Prepares the transaction by sending data to both microservices
fn prepare_transaction(
    mut wallet_stream: TcpStream,
    mut order_stream: TcpStream,
    account: u32,
    amount: u32,
    user_id: u32,
    items: &Vec<u32>,
) -> Result<(TcpStream, TcpStream), ()> {
    // Send data to wallet microservice
    if let Err(e) = wallet_stream.write(&account.to_be_bytes()) {
        eprintln!("Failed to write account id to wallet micro service: {}", e);
        return Err(());
    }
    
    if let Err(e) = wallet_stream.write(&amount.to_be_bytes()) {
        eprintln!("Failed to write balance change amount to wallet micro service: {}", e);
        return Err(());
    }
    
    // Send data to order microservice
    if let Err(e) = order_stream.write(&user_id.to_be_bytes()) {
        eprintln!("Failed to write user id to order micro service: {}", e);
        return Err(());
    }
    
    let items_count = items.len() as u32;
    println!("Items count: {}", items_count);
    
    if let Err(e) = order_stream.write(&items_count.to_be_bytes()) {
        eprintln!("Failed to write amount of items to order micro service: {}", e);
        return Err(());
    }
    
    // Send each item to order microservice
    for item in items {
        if let Err(e) = order_stream.write(&item.to_be_bytes()) {
            eprintln!("Failed to write item to order microservice: {}", e);
            return Err(());
        }
    }
    
    Ok((wallet_stream, order_stream))
}

/// Gets responses from microservices after prepare phase
fn get_prepare_responses(
    mut wallet_stream: TcpStream,
    mut order_stream: TcpStream,
) -> Result<(TcpStream, TcpStream, u8, u8), ()> {
    let mut wallet_response = [0u8];
    let mut order_response = [0u8];
    
    // Read response from wallet microservice
    if let Err(e) = wallet_stream.read(&mut wallet_response) {
        eprintln!("Failed to read wallet microservice \"ready to commit\" message: {}", e);
        return Err(());
    }
    
    // Read response from order microservice
    if let Err(e) = order_stream.read(&mut order_response) {
        eprintln!("Failed to read order microservice \"ready to commit\" message: {}", e);
        return Err(());
    }
    
    Ok((wallet_stream, order_stream, wallet_response[0], order_response[0]))
}

/// Logs the responses from microservices
fn log_microservice_responses(wallet_response: u8, order_response: u8) {
    print!("Wallet response: {}", wallet_response);
    if wallet_response < RESPONSE_DEFINITIONS.len() as u8 {
        println!(" ({})", RESPONSE_DEFINITIONS[wallet_response as usize]);
    } else {
        println!();
    }
    
    print!("Order response: {}", order_response);
    if order_response < RESPONSE_DEFINITIONS.len() as u8 {
        println!(" ({})", RESPONSE_DEFINITIONS[order_response as usize]);
    } else {
        println!();
    }
}

/// Sends commit message to a microservice with retry
fn send_commit_message(stream: &mut TcpStream) -> Result<(), ()> {
    let commit_message = 1u32;
    
    // Try to send commit message
    if let Err(e) = stream.write(&commit_message.to_be_bytes()) {
        eprintln!("Microservice failed to commit: {}", e);
        
        // Retry once
        if let Err(e) = stream.write(&commit_message.to_be_bytes()) {
            eprintln!("Microservice failed to commit twice. Contact system administrator. Error: {}", e);
            return Err(());
        }
    }
    
    Ok(())
}

/// Rolls back the transaction on both microservices
fn rollback(mut order_stream: TcpStream, mut wallet_stream: TcpStream) {
    println!("Rolling back transactions");
    
    let rollback_message = 2u32;
    let mut wallet_rolledback = false;
    let mut order_rolledback = false;
    let mut attempts = 0;
    
    // Try to send rollback message to wallet service
    if let Ok(_) = wallet_stream.write(&rollback_message.to_be_bytes()) {
        wallet_rolledback = true;
    } else {
        eprintln!("Wallet microservice rollback write failed");
    }
    
    // Try to send rollback message to order service
    if let Ok(_) = order_stream.write(&rollback_message.to_be_bytes()) {
        order_rolledback = true;
    } else {
        eprintln!("Order microservice rollback write failed");
    }
    
    // Retry sending rollback messages if needed
    while attempts < MAX_RETRY_ATTEMPTS && (!wallet_rolledback || !order_rolledback) {
        attempts += 1;
        
        if !wallet_rolledback {
            if let Ok(_) = wallet_stream.write(&rollback_message.to_be_bytes()) {
                wallet_rolledback = true;
            } else {
                eprintln!("Wallet microservice rollback retry #{} failed", attempts);
            }
        }
        
        if !order_rolledback {
            if let Ok(_) = order_stream.write(&rollback_message.to_be_bytes()) {
                order_rolledback = true;
            } else {
                eprintln!("Order microservice rollback retry #{} failed", attempts);
            }
        }
        
        if wallet_rolledback && order_rolledback {
            println!("Rollback successful");
            return;
        }
    }
    
    eprintln!("CRITICAL: Rollback failed after {} attempts!", MAX_RETRY_ATTEMPTS);
}

/// Reads and parses an HTTP request
fn read_http_request(client_stream: &TcpStream) -> HttpRequestResult {
    let mut reader = BufReader::new(client_stream);
    
    // Read the first line of the HTTP request
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return HttpRequestResult::BadRequest;
    }
    
    let request_parts: Vec<&str> = request_line.split_whitespace().collect();
    if request_parts.len() < 2 {
        return HttpRequestResult::BadRequest;
    }
    
    println!("{}", request_line.trim());
    
    // Handle GET requests
    if request_parts[0] == "GET" {
        match request_parts[1] {
            "/" => return HttpRequestResult::ServeFile("client/index.html".to_string()),
            "/favicon.ico" => return HttpRequestResult::ServeFile("client/rust-logo.png".to_string()),
            _ => return HttpRequestResult::NotFound,
        }
    }
    
    // Handle POST requests
    if request_parts[0] == "POST" {
        if request_parts[1] != "/purchase" {
            return HttpRequestResult::NotFound;
        }
        
        // Read headers and find Content-Length
        let mut content_length = 0;
        for line in reader.by_ref().lines() {
            let line = match line {
                Ok(line) => line,
                Err(_) => return HttpRequestResult::BadRequest,
            };
            
            println!("{}", line);
            
            // Parse Content-Length header
            if line.to_lowercase().starts_with("content-length:") {
                if let Ok(len) = line[16..].trim().parse() {
                    content_length = len;
                }
            }
            
            // Empty line indicates end of headers
            if line.is_empty() {
                break;
            }
        }
        
        // Read request body
        let mut body = vec![0; content_length];
        if let Err(_) = reader.read_exact(&mut body) {
            return HttpRequestResult::BadRequest;
        }
        
        // Convert body to string
        let body_string = match String::from_utf8(body) {
            Ok(s) => s,
            Err(_) => return HttpRequestResult::BadRequest,
        };
        
        println!("Body: {}", body_string);
        
        // Parse JSON
        match serde_json::from_str::<Order>(&body_string) {
            Ok(order) => {
                println!("JSON read successful");
                return HttpRequestResult::Purchase {
                    account: order.account,
                    amount: order.amount,
                    user_id: order.user_id,
                    items: order.items,
                };
            },
            Err(e) => {
                eprintln!("JSON serialization failed: {}", e);
                return HttpRequestResult::BadRequest;
            }
        }
    }
    
    HttpRequestResult::NotFound
}
