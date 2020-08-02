use std::{
    env,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

fn main() {
    let wait = || {
        thread::sleep(Duration::from_secs(1));
    };

    let args: Vec<String> = env::args().collect();

    let port = &args[1];

    let host = format!("{}:{}", "localhost", port);
    let server = TcpListener::bind(host).expect("Cannot listen to selected port");

    let mut clients = vec![];

    server
        .set_nonblocking(true)
        .expect("Failed to initialize non-blocking TCP listener");

    println!("Start listening on port {}", port);

    let (sender_msg, receiver_msg) = crossbeam::unbounded();
    let (sender_cli, receiver_cli) = crossbeam::unbounded();

    // listening thread spawned so as not to block
    thread::spawn(move || loop {
        if let Ok((tcp_stream, address)) = server.accept() {
            println!("Connection established: {}", address);

            sender_cli
                .send((
                    tcp_stream.try_clone().expect("Cannot clone handle"),
                    address.clone(),
                ))
                .expect("Cannot send connections over channel");

            let sender_msg = sender_msg.clone();
            let mut reader = BufReader::new(tcp_stream);

            thread::spawn(move || loop {
                let mut buf = String::new();

                // this will always read until '\n'
                match reader.read_line(&mut buf) {
                    Ok(_) => {
                        let buf = buf.trim();
                        println!("Received '{}' from {}", buf, address);
                        sender_msg
                            .send(format!("{}: {}", address, buf))
                            .expect("Channel broken");
                    }
                    Err(_) => {
                        sender_msg
                            .send(format!("Closing connection to: {}", address))
                            .expect("Channel broken");
                        break;
                    }
                }

                wait();
            });
        }
    });

    // sending thread
    loop {
        if let Ok(conn) = receiver_cli.try_recv() {
            clients.push(conn);
        }

        if let Ok(msg) = receiver_msg.try_recv() {
            let msg = format!("{}\n", msg);
            // update number of clients
            println!("Sending to {:?}", clients);
            clients = clients
                .into_iter()
                .filter_map(|(mut cli, addr)| match cli.write(msg.as_bytes()) {
                    Ok(_) => Some((cli, addr)),
                    Err(_) => {
                        println!("Connection to {} closed", addr);
                        None
                    }
                })
                .collect();
        }

        wait();
    }

    // unreachable!();
}
