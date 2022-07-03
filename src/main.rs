// kunevi wrote this.

use tokio::{
    net::TcpListener,
    sync::broadcast, io::{BufReader, AsyncWriteExt, AsyncBufReadExt},
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    let (tx, _rx) = broadcast::channel(10);

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        match result {
                            Ok(0..=2) => {
                                // client leaves the chat server
                                let msg = format!("{:?} has left the chat! \r\n", addr);
                                print!("{msg}");
                                tx.send((addr, msg)).unwrap();
                                break;
                            }

                            Ok(_) => {
                                // theres a message
                                let msg = format!("{:?}: {}\r\n", addr, line.trim());
                                print!("{msg}");
                                tx.send((addr, msg)).unwrap();
                            }

                            Err(e) => {
                                // theres an error
                                println!("Error: {}", e);
                                break;
                            }
                        };
                        line.clear();
                    }

                    result = rx.recv() => {
                        let (other_addr, msg) = result.unwrap();
                        if other_addr == addr {
                            writer.write_all(format!("You said: {}", msg.split_once(": ").unwrap().1).as_bytes()).await.unwrap();
                        } else {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        } 
                    }
                }
            }
        });

    }
}