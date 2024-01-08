use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use anyhow::Result;

use crate::constants::MAX_MESSAGE_SIZE;

fn handle_client(
    stream: &mut TcpStream,
    address: SocketAddr,
    sender: Sender<String>,
) -> Result<(), ()> {
    println!("Received connection from {address}");

    let mut buffer = [0u8; MAX_MESSAGE_SIZE];

    loop {
        let n = stream.read(&mut buffer).map_err(|_| {
            eprintln!("Failed to read from {address}");
        })?;

        if n == 0 {
            return Ok(());
        }

        let message = String::from_utf8(buffer[..n].to_vec()).map_err(|_| {
            eprintln!("Failed to create utf8-string from {address}");
        })?;

        sender.send(message).map_err(|_| {
            eprintln!("Couldn't send message through mpsc-channel");
        })?;
    }
}

fn broadcast(receiver: Receiver<String>, clients: Arc<Mutex<Vec<TcpStream>>>) {
    loop {
        let message = receiver.recv().expect("failed to read from mpsc-channel");

        let clients = clients.lock().unwrap();

        for mut client in clients.iter() {
            client
                .write_all(&message.as_bytes())
                .expect("failed to write to a client");
        }
    }
}

pub struct Server;

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self) -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:8080").expect("failed to bind TcpListener");
        println!("Server running on port 8080");

        let (sender, receiver) = mpsc::channel::<String>();

        let clients = Arc::new(Mutex::new(Vec::<TcpStream>::new()));

        let clients2 = clients.clone();
        thread::spawn(move || {
            broadcast(receiver, clients2);
        });

        for stream in listener.incoming() {
            let mut stream = stream.expect("failed to accept incoming connection");
            let address = stream.peer_addr().expect("unable to get peer address");

            let sender = sender.clone();

            let mut client_list = clients.lock().expect("failed to acquire lock for clients");
            client_list.push(stream.try_clone().expect("failed to clone TcpStream"));
            let clients2 = clients.clone();

            thread::spawn(move || {
                match handle_client(&mut stream, address, sender) {
                    Ok(()) => {}
                    Err(()) => eprintln!("Error while handling {address}. Closing connection..."),
                }
                let mut clients = clients2.lock().expect("failed to acquire lock for clients");
                let index = clients
                    .iter()
                    .enumerate()
                    .find(|(_, x)| x.peer_addr().unwrap() == stream.peer_addr().unwrap())
                    .expect("failed to find client in clients")
                    .0;
                clients.remove(index);
            });
        }

        Ok(())
    }
}
