use crossbeam::TryRecvError;
use std::{
    env,
    io::{self, BufRead, BufReader, Write},
    net::TcpStream,
    thread,
    time::Duration,
};

fn main() {
    let wait = || {
        thread::sleep(Duration::from_secs(1));
    };

    let args: Vec<String> = env::args().collect();

    let host = format!("{}:{}", "localhost", &args[1]);
    let mut client = TcpStream::connect(host).expect("Port not available");

    client
        .set_nonblocking(true)
        .expect("Failed to initialize non-blocking Tcp stream");

    let (sender_in, receiver_in) = crossbeam::unbounded();
    let (sender_out, receiver_out) = crossbeam::unbounded();

    let stdin = io::stdin();

    // input thread
    thread::spawn(move || loop {
        let mut buf = String::new();
        print!("Write a message:");
        stdin
            .read_line(&mut buf)
            .expect("Unable to read from stdin");
        sender_in.send(buf).expect("Input channel broken");

        wait();
    });

    // output thread
    thread::spawn(move || loop {
        if let Ok(msg) = receiver_out.try_recv() {
            println!("Message exchanged: {}", msg);
        }

        wait();
    });

    // client server thread
    let mut reader = BufReader::new(client.try_clone().expect("Cannot clone Tcp stream"));
    loop {
        let mut buf = String::new();

        match receiver_in.try_recv() {
            Ok(msg) => {
                if msg == ":quit" {
                    break;
                } else {
                    let msg = format!("{}\n", msg.trim());
                    client.write(msg.as_bytes()).expect("Server is down");
                }
            }
            Err(TryRecvError::Empty) => {
                continue;
            }
            Err(TryRecvError::Disconnected) => {
                break;
            }
        }

        if let Ok(size) = reader.read_line(&mut buf) {
            if size != 0 {
                sender_out.send(buf).expect("Output channel broken");
            }
        }

        wait();
    }

    println!("Program Exit");
}
