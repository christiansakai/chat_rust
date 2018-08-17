use std::env;
use std::thread;
use std::time::Duration;
use std::net::{TcpListener, TcpStream};
use std::io::{self, Read, Write, ErrorKind};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

const LOCAL: &'static str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "server" {
        println!("Initialize as Server");
        server();
    } else {
        println!("Initialize as Client");
        client();
    }
}

fn sleep() {
    thread::sleep(Duration::from_millis(100));
}

fn server() {
    let server = TcpListener::bind(LOCAL)
        .expect("Server: listener failed to bind");

    server
        .set_nonblocking(true)
        .expect("Server: failed to initialize non-blocking");

    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let mut clients: Vec<TcpStream> = vec![];

    loop {
        if let Ok((mut socket, address)) = server.accept() {
            println!("Server: client {} connected", address);

            let client = socket.try_clone()
                .expect(&format!("Server: failed to clone client {}", address));
            clients.push(client);

            let tx = tx.clone();

            thread::spawn(move || loop {
                let mut buffer = vec![0; MSG_SIZE];

                // Server handler thread receives message from a client
                match socket.read_exact(&mut buffer) {
                    Ok(_) => {
                        let message_bytes: Vec<_> = buffer.into_iter()
                            .take_while(|&x| x != 0)
                            .collect();

                        let message = String::from_utf8(message_bytes)
                            .expect("Server: invalid utf8 message");

                        println!("Server: received from client {}, message \"{}\"", address, message);

                        // Server handler thread sends received message to server main thread
                        tx.send(message)
                        .expect("Server: failed to send message to rx");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Server: closing connection with client {}", address);
                        break;
                    },
                }

                sleep();
            });
        }

        // Server main thread receives the received message from server handler thread
        if let Ok(message) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut buffer = message.clone().into_bytes();
                    buffer.resize(MSG_SIZE, 0);

                    client.write_all(&buffer)
                        .map(|_| client)
                        .ok()
                })
                .collect();
        }

        sleep();
    }
}

fn client() {
    let mut client = TcpStream::connect(LOCAL)
        .expect("Client: failed to connect to server"); 

    client
        .set_nonblocking(true)
        .expect("Client: failed to initiate non-blocking");

    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    thread::spawn(move || loop {
        let mut buffer = vec![0; MSG_SIZE];

        // Client handler thread receives message from the server
        match client.read_exact(&mut buffer) {
            Ok(_) => {
                let message_bytes: Vec<_> = buffer
                    .into_iter()
                    .take_while(|&x| x != 0)
                    .collect();

                println!("Client: message received {:?}", message_bytes);
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Client: connection with server was severed");
                break;
            }
        }

        // Client handler thread receives user messages from client main thread
        match rx.try_recv() {
            Ok(message) => {
                let mut buffer = message.clone().into_bytes();
                buffer.resize(MSG_SIZE, 0);

                client.write_all(&buffer)
                    .expect("Client: writing to socket failed");

                println!("Client: message sent {:?}", message);
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        sleep();
    });

    // Client main thread receives messages from user and sends it to client handler thread
    println!("Write a message:");
    loop {
        let mut buffer = String::new();

        io::stdin()
            .read_line(&mut buffer)
            .expect("Client: reading from stdin failed");

        let message = buffer.trim().to_string();

        if message == ":quit" || tx.send(message).is_err() {
            break;
        }
    }

    println!("Bye!");
}




//     println!("Write a message:");
//     loop {
//         let mut buff = String::new();
//         io::stdin()
//             .readline(&mut buff)
//             .expect("reading from stdin failed");
//         let msg = buff.trim().to_string();

//         if msg == ":quit" || tx.send(msg).is_err() {
//             break;
//         }
//     }

//     println!("bye bye!");
// }
