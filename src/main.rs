use std::io;
use std::net;
use std::thread;
use std::sync::Arc;
use std::time::Duration;
use get_if_addrs::{get_if_addrs, IfAddr, Ifv4Addr, Ifv6Addr};
use structopt::StructOpt;

fn main() {
    let opts_pre = Opts::from_args();
    let opts_a = Arc::new(opts_pre);
    loop {
        let mut sender4: Vec<Ifv4Addr> = Vec::new();
        let mut sender6: Vec<Ifv6Addr> = Vec::new();
        let mut msg = String::new();
        let (port0, port1, port2) = (1020, 1021, 1022);
        let opts = opts_a.clone();
        // msg_arc.clear();
        for iface in get_if_addrs().unwrap() {
            match iface.addr {
                IfAddr::V4(Ifv4Addr{
                    broadcast: None, ..
                }) => (),
                IfAddr::V4(Ifv4Addr{ ip, .. }) if ip == net::Ipv4Addr::LOCALHOST => (),
                IfAddr::V4(addr) => {
                    println!("{:?}", addr);
                    // String::from('\n');
                    msg.push_str(&(iface.name + " " + &addr.ip.to_string() + "\n"));
                    // msg = (*msg) + "\n" + &iface.name + " " + &addr.ip.to_string();
                    if (&opts).multicast4 {
                        sender4.push(addr.clone());
                    }
                    if (&opts).broadcast4 {
                        sender4.push(addr.clone());
                    }
                },
                IfAddr::V6(Ifv6Addr{ ip, .. }) if ip == net::Ipv6Addr::LOCALHOST || ip.is_loopback() || ip.segments()[0] == 0xfe80 => (),
                IfAddr::V6(addr) => {
                    println!("{:?}", addr);
                    msg.push_str(&(iface.name + " " + &addr.ip.to_string() + "\n"));
                    if (&opts).multicast6 {
                        sender6.push(addr);
                    }
                }
            }
        }
        // let (sender4, sender6, msg) = (*&m_sender4, *&m_sender6, *&msg);
        let (sender4_arc, sender6_arc, msg_arc) = (Arc::new(sender4), Arc::new(sender6), Arc::new(msg));
        // let (handle_m4, handle_b4, handle_m6);
        // let mut handle : [Option<_>; 3];
        let handle = vec![
            if opts_a.multicast4 {
                let (senders, msg, opts) = (sender4_arc.clone(), msg_arc.clone(), opts_a.clone());
                Some( thread::spawn(move|| {
                    for sender in &senders[..] {
                        ipv4_multicast(
                            &(sender.ip, port0),
                            &(opts.group4, opts.multiport4),
                            opts.ttl4,
                            (*msg).as_ref()
                        ).expect((sender.ip.to_string() + port0.to_string().as_ref()).as_ref());
                    }
                }))
            } else {None},
            if opts.broadcast4 {
                let (senders, msg, opts) = (sender4_arc.clone(), msg_arc.clone(), opts_a.clone());
                Some( thread::spawn(move|| {
                    for sender in &senders[..] {
                        ipv4_broadcast(
                            &(sender.ip, port1),
                            &(sender.broadcast.unwrap_or(net::Ipv4Addr::BROADCAST), opts.broadport4),
                            (*msg).as_ref()
                        ).expect((sender.ip.to_string() + port1.to_string().as_ref()).as_ref());
                    }
                }))
            } else {None},
            if opts.multicast6 {
                let (senders, msg, opts) = (sender6_arc.clone(), msg_arc.clone(), opts_a.clone());
                // let senders_arc = sender6.clone();
                Some( thread::spawn(move|| {
                    for sender in &senders[..] {
                        ipv6_multicast(
                            (sender.ip, port2),
                            (opts.group6, opts.multiport6),
                            (*msg).as_ref()
                        ).expect((sender.ip.to_string() + port2.to_string().as_ref()).as_ref());
                    }
                }))
            } else {None}
        ];
        for mut h in handle {
            if let Some(v) = h.take() {
                v.join().ok();
                // if let Some(e) = v.join().err() {
                //     std::fs::File::create("err.log").unwrap().write(e.to_string());
                //     std::io::stderr();
                // }
            }
        }
        // &handle[0].unwrap().join().ok();
        thread::sleep(Duration::from_secs(opts_a.interval.into()));
    }
    // Ok(())
}

fn ipv4_multicast<A: net::ToSocketAddrs, B: net::ToSocketAddrs>(addr: &A, multigroup: &B, ttl4:u32, buf: &[u8]) -> io::Result<usize> {
    let sock = net::UdpSocket::bind(addr).expect("bind");
    sock.set_multicast_ttl_v4(ttl4).expect("set ttl");
    sock.set_multicast_loop_v4(true)?;
    sock.send_to(buf, multigroup)
}

fn ipv4_broadcast<A: net::ToSocketAddrs, B: net::ToSocketAddrs>(addr: A, broadaddr: B, buf: &[u8]) -> io::Result<usize> {
    let sock = net::UdpSocket::bind(addr).expect("bind");
    sock.set_broadcast(true)?;
    sock.send_to(buf, broadaddr)
}

fn ipv6_multicast<A: net::ToSocketAddrs, B: net::ToSocketAddrs>(addr: A, multigroup: B, buf: &[u8]) -> io::Result<usize> {
    let sock = net::UdpSocket::bind(addr).expect("bind");
    sock.set_multicast_loop_v6(true)?;
    sock.send_to(buf, multigroup)
}

#[derive(StructOpt, Debug)]
struct Opts {
    // cast interval
    #[structopt(short, default_value = "6")]
    interval: u8,

    // ipv4 multicast
    #[structopt(long = "m4")]
    multicast4: bool,
    // multicast group for ipv4
    #[structopt(long = "addr4", default_value = "224.0.2.42")]
    group4: net::Ipv4Addr,
    // multicast port for ipv4
    #[structopt(long = "mp4", default_value = "1010")]
    multiport4: u16,
    // multicast ttl for ipv4
    #[structopt(long = "t4", default_value = "15")]
    ttl4: u32,

    // ipv4 broadcast
    #[structopt(long = "b4")]
    broadcast4: bool,
    // broadcast port for ipv4
    #[structopt(long = "bp4", default_value = "1010")]
    broadport4: u16,

    // ipv6 multicast
    #[structopt(long = "m6")]
    multicast6: bool,
    // multicast group for ipv6
    #[structopt(long = "addr6", default_value = "ff1e::2:a")]
    group6: net::Ipv6Addr,
    // multicast port for ipv6
    #[structopt(long = "mp6", default_value = "1010")]
    multiport6: u16,
    // multicast ttl for ipv6
    #[structopt(long = "t6", default_value = "15")]
    ttl6: u32,
}
