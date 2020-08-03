use std::{
    collections::LinkedList,
    env,
    io::{BufRead, BufReader, BufWriter, ErrorKind, Write},
    net::TcpListener,
    thread::{self, JoinHandle},
    time::Duration,
};

fn main() {
    let wait = || {
        thread::sleep(Duration::from_secs(1));
    };

    let args: Vec<String> = env::args().collect();

    let port = args.get(1).expect("Index out of bounds");

    let host = format!("{}:{}", "localhost", port);
    let server = TcpListener::bind(host).expect("Cannot listen to selected port");

    let (snd_io, rcv_io) = crossbeam::unbounded();
    let (snd_inter, rcv_inter) = crossbeam::unbounded();

    server
        .set_nonblocking(true)
        .expect("Failed to initialize non-blocking TCP listener");

    println!("Start listening on port {}", port);

    let mut handles = Vec::new();

    // IO thread
    handles.push(thread::spawn(move || loop {
        if let Ok(msg) = rcv_io.try_recv() {
            println!("Broadcasting: {}", msg);
        }

        wait();
    }));

    // communication thread
    handles.push(thread::spawn(move || {
        let mut connections = LinkedList::new();
        loop {
            if let Ok((conn, addr)) = server.accept() {
                let mut conn_reader =
                    BufReader::new(conn.try_clone().expect("Cannot clone connection"));
                let conn_writer =
                    BufWriter::new(conn.try_clone().expect("Cannot clone connection"));
                connections.push_front(conn_writer);

                snd_io
                    .send(format!("Connected to {}", addr))
                    .expect("Cannot send through channel");

                let snd_inter = snd_inter.clone();

                let snd_io = snd_io.clone();

                thread::spawn(move || {
                    loop {
                        let mut msg = String::new();

                        match conn_reader.read_line(&mut msg) {
                            Err(err) if err.kind() == ErrorKind::WouldBlock => (),
                            Ok(0) | Err(_) => break,
                            Ok(_) => snd_inter
                                .send(format!("{}: {}", addr, msg))
                                .expect("Cannot send through channel"),
                        }

                        wait();
                    }
                    snd_io
                        .send(format!("Connection to {} closed", addr))
                        .expect("Cannot send through channel");
                });
            }

            if let Ok(msg) = rcv_inter.try_recv() {
                let msg = format!("{}\n", msg.trim());
                connections = connections
                    .into_iter()
                    .filter_map(|mut conn| match conn.write(msg.as_bytes()) {
                        Err(err) if err.kind() == ErrorKind::WouldBlock => Some(conn),
                        Ok(0) | Err(_) => None,
                        Ok(_) => {
                            conn.flush().expect("Cannot flush");
                            Some(conn)
                        }
                    })
                    .collect();

                snd_io.send(msg).expect("Cannot send through channel");
                println!("Remaining connections: {:?}", connections);
            }

            wait();
        }
    }));

    handles
        .into_iter()
        .map(JoinHandle::join)
        .map(Result::unwrap)
        .for_each(|_| ());
}
