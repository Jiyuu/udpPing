#[macro_use]
extern crate clap;

extern crate rand;
extern crate time;


use clap::{Arg, App, SubCommand, ArgGroup};
use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};
use rand::{thread_rng, Rng};
use std::time::Duration as stdDuration;
use time::{PreciseTime, Duration};
use std::thread;

fn main() {
    let matches = App::new("Simple Udp ping")
        .version(crate_version!())
        .author(crate_authors!())
        .about("listens to pings over udp and replies to them")
        .arg(Arg::with_name("target")
            .help("the target of the ping attampt")
            .value_name("URL:PORT")
            .required_unless("daemon"))
        .arg(Arg::with_name("interval")
            .help("how long in ms to wait between ping messages")
            .value_name("DELAYms")
            .default_value("100"))
        .arg(Arg::with_name("daemon")
            .short("d")
            .long("daemon")
            .help("runs in daemon mode, listening to ping requests and responding to them")
            .value_name("PORT")
            .takes_value(true))

        .get_matches();


    //let target = matches.value_of("target");

    let daemon_mode = matches.is_present("daemon");

    if (!daemon_mode) {
        let target = matches.value_of("target").unwrap().to_socket_addrs().unwrap().next().unwrap();
        let interval: u16 = value_t_or_exit!(matches,"interval",u16);
        println!("pinging {}", target);
        ping_target(target, interval);
    } else {
        let daemon_port: u16 = value_t_or_exit!(matches,"daemon",u16);
        run_daemon(daemon_port);
    }
}

fn run_daemon(port: u16) {
    let socket = UdpSocket::bind(SocketAddr::new("0.0.0.0".parse().unwrap(), port)).unwrap();
            socket.set_nonblocking(false);
    let mut data: [u8; 65535] = [0; 65535];
    while true {
        let recv_result = socket.recv_from(&mut data);
        if let Ok((length, addr)) = recv_result {
            if length == 100 && data[0] == 1 {
                //maybe an actual packet

                data[0] = 0;
                socket.send_to(&data[0..length], addr).unwrap();
            }
        }
    }
}

fn ping_target(target: SocketAddr, interval: u16)
{
    let addr: SocketAddr = "0.0.0.0:44214".parse().unwrap();
    let socket = UdpSocket::bind(addr).unwrap();

    socket.connect(target).unwrap();
    loop{
        let mut rng = thread_rng();
        let mut data: [u8; 100] = [0; 100];
        {
            let slice = &mut data[10..];
            rng.fill(slice);
            data[0] = 1;
        }
        socket.send(&data).unwrap();
        let wait_start_time = PreciseTime::now();

        let received_data = socket.recv(&mut data).unwrap();
        let possible_end_time = PreciseTime::now();
        if (received_data != data.len())
        {
            println!("malformed response!");
        } else {
            //TODO:do more verification
            let latency = wait_start_time.to(possible_end_time);
            println!("ping time {}", latency.num_milliseconds());
        }

        thread::sleep(stdDuration::from_millis(interval.into()));
    }
}

