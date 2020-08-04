use anyhow::Result;
use std::{
    env,
    io::{self, BufRead, BufReader, BufWriter, ErrorKind, Write},
    net::TcpStream,
    thread::{self, JoinHandle},
    time::Duration,
};

fn main() -> Result<()> {
    let wait = || {
        thread::sleep(Duration::from_secs(1));
    };

    let args: Vec<String> = env::args().collect();

    let host = format!("{}:{}", "localhost", &args[1]);
    let client = TcpStream::connect(host)?;

    client.set_nonblocking(true)?;

    let (snd_input, rcv_input) = crossbeam::unbounded();
    let (snd_output, rcv_output) = crossbeam::unbounded();

    let mut stdin_read = BufReader::new(io::stdin());
    let mut client_write = BufWriter::new(client.try_clone()?);
    let mut client_read = BufReader::new(client.try_clone()?);

    let mut handles = Vec::new();

    // IO thread
    handles.push(thread::spawn(move || -> Result<()> {
        loop {
            println!("Write a message:");

            let mut msg = String::new();
            stdin_read.read_line(&mut msg)?;

            snd_input.send(msg)?;

            if let Ok(msg) = rcv_output.try_recv() {
                println!("Exchanged: {}", msg);
            }

            wait();
        }
    }));

    // communication thread
    handles.push(thread::spawn(move || -> Result<()> {
        loop {
            if let Ok(msg) = rcv_input.try_recv() {
                let msg = format!("{}\n", msg.trim());
                client_write.write_all(msg.as_bytes())?;
                client_write.flush()?;
            }

            let mut msg = String::new();

            match client_read.read_line(&mut msg) {
                Err(err) if err.kind() == ErrorKind::WouldBlock => (),
                Ok(0) | Err(_) => break Ok(()),
                Ok(_) => snd_output.send(msg)?,
            }

            wait();
        }
    }));

    handles
        .into_iter()
        .map(JoinHandle::join)
        .map(Result::unwrap)
        .for_each(|_| ());

    Ok(())
}
