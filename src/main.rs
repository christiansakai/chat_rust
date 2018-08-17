use std::io::{self, ErrorKind, Read, Write};
use std::net::{TcpStream, TcpListener};
use std::sync::mpsc::{self, Sender, Receiver, TryRecvError} ;
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
}

fn sleep() {
    thread::sleep(Duration::from_millis(100));
}

fn server() {
    let server = TcpListener::bind(LOCAL)
        .expect("listener failed to bind");

    server.set_nonblocking(true)
        .expect("failed to initialize non-blocking");

    let mut clients: Vec<_> = vec![];
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} conneced", addr);

            let tx = tx.clone();
            clients.push(socket.try_clone().expect("failed to clone client"));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg: Vec<_> = buff
                            .into_iter()
                            .take_while(|&x| x != 0)
                            .collect();

                        let msg = String::from_utf8(msg)
                            .expect("invalid utf8 message");

                        println!("{}: {:?}", addr, msg);
                        tx.send(msg).expect("failed to send msg to rx");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("closing connection with: {}", addr);
                    },
                }

                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter()
                .filter_map(|mut client| {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);

                    client.write_all(&buff).map(|_| client).ok()
                }).collect();
        }

        sleep();
    }
}

fn client() {
    let mut client = TcpStream::connect(LOCAL).expect("stream failed to connect");
    client.set_nonblocking(true).expect("failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter()
                    .take_while(|&x| x != 0)
                    .collect()::<Vec<_>>();

                println!("message recv {:?}", msg);
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("connection with server was severed");
                break;
            },
        });

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("writing to socket failed");
                println!("message sent {:?}", msg);
            }, 
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a message:");
    loop {
        let mut buff = String::new();
        io::stdin().readline(&mut buff).expect("reading from stdin failed");
        let msg = buff.trim().to_string();

        if msg == ":quit" || tx.send(msg).is_err() { break }
    }

    println!("bye bye!");




}
