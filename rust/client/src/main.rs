use std::{
    env,
    io::{self, BufRead, BufReader, BufWriter, ErrorKind, Write},
    net::TcpStream,
    thread::{self, JoinHandle},
    time::Duration,
};

fn main() {
    let wait = || {
        thread::sleep(Duration::from_secs(1));
    };

    let args: Vec<String> = env::args().collect();

    let host = format!("{}:{}", "localhost", &args[1]);
    let client = TcpStream::connect(host).expect("Port not available");

    client
        .set_nonblocking(true)
        .expect("Failed to initialize non-blocking Tcp stream");

    let (snd_input, rcv_input) = crossbeam::unbounded();
    let (snd_output, rcv_output) = crossbeam::unbounded();

    let mut stdin_read = BufReader::new(io::stdin());
    let mut client_write = BufWriter::new(client.try_clone().expect("Cannot clone client"));
    let mut client_read = BufReader::new(client.try_clone().expect("Cannot clone client"));

    let mut handles = Vec::new();

    // IO thread
    handles.push(thread::spawn(move || loop {
        println!("Write a message:");

        let mut msg = String::new();
        stdin_read
            .read_line(&mut msg)
            .expect("Cannot read from stdin");

        snd_input.send(msg).expect("Cannot send through channel");

        if let Ok(msg) = rcv_output.try_recv() {
            println!("Exchanged: {}", msg);
        }

        wait();
    }));

    // communication thread
    handles.push(thread::spawn(move || loop {
        if let Ok(msg) = rcv_input.try_recv() {
            let msg = format!("{}\n", msg.trim());
            client_write
                .write(msg.as_bytes())
                .expect("Cannot write to server");
            client_write.flush().expect("Cannot flush");
        }

        let mut msg = String::new();

        match client_read.read_line(&mut msg) {
            Err(err) if err.kind() == ErrorKind::WouldBlock => (),
            Ok(0) | Err(_) => break,
            Ok(_) => snd_output.send(msg).expect("Cannot send through channel"),
        }

        wait();
    }));

    handles
        .into_iter()
        .map(JoinHandle::join)
        .map(Result::unwrap)
        .for_each(|_| ());
}
