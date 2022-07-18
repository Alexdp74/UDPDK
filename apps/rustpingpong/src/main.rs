use rustudpdk::*;
use std::env;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

const PING_PORT: u16 = 10000;
const PONG_PORT: u16 = 10001;
const IP_PONG: [u8; 4] = [10, 10, 1, 2];

fn ping_body(app_alive: Arc<Mutex<bool>>) {
    println!("PING mode");

    let s = UDPDK::socket(AF_INET as i32, SOCK_DGRAM, 0);
    s.bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        PING_PORT,
    ));

    let dst = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(
            IP_PONG[0], IP_PONG[1], IP_PONG[2], IP_PONG[3],
        )),
        PONG_PORT,
    );

    loop {
        let a = app_alive.lock().unwrap();
        if !*a {
            break;
        }

        let now = SystemTime::now();
        let serialized_time: Vec<u8> = bincode::serialize(&now).unwrap();

        let _ = s.sendto(&serialized_time[..], 0, dst);

        let mut buf = [0u8; 100];
        let retval = s.recvfrom(&mut buf, 0);
        //let retval = s.recvfrom(&mut buf, MSG_DONTWAIT); // non-blocking call
        match retval {
            Ok((sz, _from)) => {
                let deserialized_time: SystemTime = bincode::deserialize(&buf[..sz]).unwrap();
                match deserialized_time.elapsed() {
                    Ok(elapsed) => {
                        println!("ping latency {} usec", elapsed.as_micros());
                    }
                    Err(e) => eprintln!("Error {}", e),
                }
            }
            Err(e) if e.kind() != ErrorKind::WouldBlock => {
                eprintln!("Error {}", e);
            }
            _ => (),
        }

        thread::sleep(Duration::new(1, 0));
    }
}

fn pong_body(app_alive: Arc<Mutex<bool>>) {
    println!("PONG mode");

    let s = UDPDK::socket(AF_INET as i32, SOCK_DGRAM, 0);
    s.bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        PONG_PORT,
    ));

    loop {
        let a = app_alive.lock().unwrap();
        if !*a {
            break;
        }

        let mut buf = [0u8; 100];
        let retval = s.recvfrom(&mut buf, 0);
        match retval {
            Ok((sz, from)) => {
                s.sendto(&buf[..sz], 0, from);
            }
            Err(e) => eprintln!("Error {}", e),
        }
    }
}

// usage: ./app -c ../../config.ini
fn main() {
    UDPDK::init(env::args().collect::<Vec<String>>());
    println!("After init");

    let app_alive = Arc::new(Mutex::new(true));

    let alive = app_alive.clone();
    ctrlc::set_handler(move || {
        {
            let mut a = alive.lock().unwrap();
            *a = false;
        }
        UDPDK::interrupt(9);
    })
    .expect("Error setting Ctrl-C handler");

    if true {
        ping_body(app_alive);
    } else {
        pong_body(app_alive);
    }

    UDPDK::interrupt(0);
    println!("After interrupt");

    UDPDK::cleanup();
    println!("After cleanup");
}
