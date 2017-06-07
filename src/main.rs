use std::env;
use std::fmt;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;

extern crate byteorder;
extern crate fern;
extern crate getopts;
use getopts::{Matches, Options};
#[macro_use]
extern crate log;
extern crate mtbl;
extern crate num_cpus;
extern crate objpool;
extern crate regex;
use regex::Regex;
extern crate time;
extern crate tinycdb;

mod kvstore;
use kvstore::KvStore;
use kvstore::cdb::new_cdb_pool;
use kvstore::mtbl::new_mtbl;

mod memcached;
use memcached::server::memcached_server;


/// A database to serve
#[derive(Debug,Clone)]
enum DbArg {
    Cdb(String),
    Mtbl(String),
}

/// A address and port to run a service on
#[derive(Debug,Clone)]
struct Listen {
    address: String,
    port: u16,
}

impl fmt::Display for Listen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.address, self.port)
    }
}

/// A service to run
#[derive(Debug,Clone)]
enum ServiceArg {
    Memcached(Listen),
}

#[derive(Debug,Clone)]
struct Args {
    db: DbArg,
    services: Vec<ServiceArg>,
    verbosity: u8,
}

fn parse_address_and_port(s: &str) -> Listen {
    Regex::new(r"^((?P<address>.*):)?(?P<port>\d+)$")
        .unwrap()
        .captures(s)
        .map(|c| {
            Listen {
                address: c.name("address").map_or("0.0.0.0", |m| m.as_str()).to_owned(),
                port: c.name("port")
                       .map(|m| {
                           u16::from_str(m.as_str())
                               .expect(&format!("error parsing port from \"{}\"", m.as_str()))
                       })
                       .unwrap(),
            }
        })
        .expect(&format!("error parsing address and port from \"{}\"", s))
}

fn parse_services(matches: &Matches) -> Vec<ServiceArg> {
    let services: Vec<ServiceArg> = vec![("memcached", ServiceArg::Memcached)]
        .iter()
        .map(|&(name, service_f)|
             matches.opt_str(name)
             .map(|s| service_f(parse_address_and_port(&s))))
        // remove Nones
        .flat_map(|o| o.into_iter())
        .collect();
    match services.len() {
        0 => panic!("no services to run!"),
        _ => services,
    }
}

fn parse_db(matches: &Matches) -> DbArg {
    let db_matchers: Vec<(&str, fn(String) -> DbArg)> = vec![("cdb", DbArg::Cdb),
                                                             ("mtbl", DbArg::Mtbl)];
    let mut dbs: Vec<DbArg> = db_matchers.iter()
        .map(|&(name, db_f)|
             matches.opt_str(name)
             .map(|s| db_f(s)))
        // remove Nones
        .flat_map(|o| o.into_iter())
        .collect();
    match dbs.len() {
        1 => dbs.pop().unwrap(),
        _ => panic!("Error: specify exactly one database file"),
    }
}

fn parse_args() -> Args {
    let mut opts = Options::new();
    opts.optopt("",
                "memcached",
                "What port (and optional address) to bind a memcached service on (default \
                 address \"0.0.0.0\")",
                "[HOST:]PORT");
    opts.optopt("", "cdb", "A CDB file to serve", "CDB");
    opts.optopt("", "mtbl", "An MTBL file to serve", "MTBL");
    opts.optflagmulti("v", "verbose", "Print more logging information");
    opts.optflag("h", "help", "Print this help text");
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f.to_string());
            exit(2);
        }
    };
    if matches.opt_present("help") {
        print!("{}", opts.usage(&format!("Usage: {} [options]", program)));
        exit(2);
    }
    if !matches.free.is_empty() {
        panic!("unexpected arguments");
    }
    Args {
        db: parse_db(&matches),
        services: parse_services(&matches),
        verbosity: matches.opt_count("verbose") as u8,
    }
}

fn setup_logger(verbosity: u8) {
    fern::Dispatch::new()
        .format(|out, message, record| {
            let t = time::now_utc();
            out.finish(format_args!("[{}.{:03}Z][{}][{}] {}",
                                    t.strftime("%FT%T").unwrap(),
                                    t.tm_nsec / 1000000, // milliseconds
                                    record.level(),
                                    record.target(),
                                    message))
        })
        .level(match verbosity {
            0 => log::LogLevelFilter::Warn,
            1 => log::LogLevelFilter::Info,
            _ => log::LogLevelFilter::Trace,
        })
        .chain(std::io::stdout())
        .apply()
        .expect("Failed to initialize global logger");
}

fn open_db(db: &DbArg) -> Arc<KvStore + Send + Sync> {
    match db {
        &DbArg::Cdb(ref f) => {
            Arc::new(new_cdb_pool(Path::new(&f),
                                  // Support a parallelism of 10 + 10 per CPU. Is
                                  // that good? It seems like a start.
                                  10 + 10 * num_cpus::get()))
        }
        &DbArg::Mtbl(ref f) => Arc::new(new_mtbl(Path::new(&f))),
    }
}

fn spawn_service(service: ServiceArg,
                 db: &DbArg,
                 kvstore: &Arc<KvStore + Send + Sync>)
                 -> thread::JoinHandle<()> {
    println!("Serving from {:?} on {:?}", db, service);
    let service = service.clone();
    let kvstore = kvstore.clone();
    thread::spawn(move || match service {
        ServiceArg::Memcached(Listen { address, port }) => {
            memcached_server(kvstore, &address, port);
        }
    })
}

fn main() {
    let Args { services, db, verbosity } = parse_args();
    setup_logger(verbosity);
    // Load the database.
    let kvstore = open_db(&db);
    // Start all services.
    let threads: Vec<thread::JoinHandle<()>> = services.into_iter()
                                                       .map(|service| {
                                                           spawn_service(service, &db, &kvstore)
                                                       })
                                                       .collect();
    // Wait on all server threads.
    for thread in threads {
        thread.join().expect("server thread failed");
    }
}
