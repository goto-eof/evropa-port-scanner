use clap::{command, Parser};
use futures::future::join_all;
use std::{
    net::{TcpStream, ToSocketAddrs},
    sync::{Arc, Mutex},
    time::Duration,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();
    scan_all(args.ip.as_str(), args.ttl).await;
}

pub async fn scan_all(ip: &str, ttl: u64) {
    let i: u16 = 0;
    let i_mutex = Arc::new(Mutex::new(i));
    let ip_mutex = Arc::new(Mutex::new(ip.to_owned()));
    loop {
        let mut handlers = Vec::new();
        for _ in 1..num_cpus::get() {
            let i_mutex = i_mutex.clone();
            let ip_mutex = ip_mutex.clone();
            let result = tokio::spawn(async move {
                let mut i = i_mutex.lock().unwrap();
                let ip = ip_mutex.lock().unwrap();
                scan(ip.to_string(), *i, ttl);
                *i = *i + 1;
            });
            handlers.push(result);
        }
        let result = join_all(handlers).await;
        result.iter().for_each(|item| {
            if item.is_err() {
                println!("Scanning error");
            }
        });
        let ii = i_mutex.lock().unwrap();
        if *ii > u16::MAX {
            break;
        }
    }
}

pub fn scan(ip: String, i: u16, ttl: u64) {
    let result = TcpStream::connect_timeout(
        &format!("{}:{}", ip, i)
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap(),
        Duration::from_millis(ttl),
    );
    if result.is_ok() {
        println!("{} opened", i);
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP addres on which port scanner should work
    #[arg(short, long)]
    ip: String,

    /// timeout of the request in millis
    #[arg(short, long, default_value_t = 1000)]
    ttl: u64,
}
