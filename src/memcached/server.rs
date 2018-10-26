use std::io::{BufReader, BufWriter, Cursor, Read};
use std::net::SocketAddr;

use byteorder::ReadBytesExt;

use super::binary::protocol::constants as binary_constants;
use kvstore::KvStore;
//use super::binary::server as binary_server;
use super::error::Error;
use super::text::server as text_server;
extern crate tokio;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

pub fn memcached_server<KV>(kvstore: KV, addr: SocketAddr)
where
    KV: KvStore,
    KV: Clone,
    KV: Send,
    KV: 'static,
{
    //let listener = TcpListener::bind(addr).expect(&format!("Failed to open {}", addr));
    let listener = TcpListener::bind(&addr).unwrap();

    // accept connections and process them, spawning a new thread for each one
    let server = listener
        .incoming()
        .for_each(move |socket| {
            let kvs = kvstore.clone();
            tokio::spawn(handle_client(kvs, socket).map_err(|e| panic!(e)));
            Ok(())
        })
        .map_err(|err| println!("accept error = {:?}", err));

    tokio::run(server);
}

fn handle_client<KV: KvStore>(
    kvstore: KV,
    mut stream: TcpStream,
) -> future::FutureResult<(), Error> {
//) -> impl Future<Item=(), Error=Error> {
    //    let first_char: u8 = try!(stream.read_u8());
    //    let fake_stream = Cursor::new(vec![first_char]);
    let addr = stream.peer_addr().unwrap();
    //    let peeked = BufReader::new(fake_stream.chain(stream.try_clone().unwrap()));
    let (raw_reader, raw_writer) = stream.split();
    let writer = BufWriter::new(raw_writer);
    //    let binary = first_char == binary_constants::REQUEST_MAGIC;
    //    let protocol_name = match binary {
    //        true => "memcached_binary",
    //        false => "memcached_text",
    //    };
    //    info!("{} connection from {}", protocol_name, addr);
    //    let result = match binary {
    //        true => binary_server::handle_client(kvstore, peeked, writer),
    //        _ => text_server::handle_client(kvstore, peeked, writer),
    //    };
    let reader = BufReader::new(raw_reader);
    let protocol_name = "memcached_text";
    let result = text_server::handle_client(kvstore, reader, writer);
    info!("{} disconnection from {}", protocol_name, addr);
    future::result(result)
}

#[cfg(test)]
mod test {
    use std::io::{Read, Write};
    use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr};
    use std::thread;

    use tokio::net::{TcpListener, TcpStream};
    use tokio::prelude::*;

    use super::super::binary::protocol::{constants, AResponse, PRead, PWrite, Request,
                                         RequestHeader, ResponseHeader};
    use kvstore::KvStore;

    /// A KvStore with one pair, {"k": "v"}
    struct DummyKvStore {}

    impl KvStore for DummyKvStore {
        fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
            if key == "k".as_bytes() {
                Some("v".as_bytes().to_vec())
            } else {
                None
            }
        }
    }

    fn make_server_conn() -> std::net::TcpStream {
        let mut listener = TcpListener::bind(&"127.0.0.1:0".parse().unwrap()).unwrap();
        let port = listener.local_addr().unwrap().port();
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let client_conn = std::net::TcpStream::connect(&socket).unwrap();
        thread::spawn(move || {
            let server = listener.incoming()
                .take(1)
                .for_each(move |socket| {
                    tokio::spawn(super::handle_client(DummyKvStore{}, socket).map_err(|_| ()));
                    Ok(())
                })
                .map_err(|err| println!("accept error = {:?}", err));
            tokio::run(server);
        });
        client_conn
    }

    #[test]
    fn test_handle_client_nonsense() {
        let mut client_stream = make_server_conn();
        // If we send nonsense, we get an error.
        client_stream.write("hihi".as_bytes()).unwrap();
        client_stream.shutdown(Shutdown::Write).unwrap();
        let mut response = String::new();
        client_stream.read_to_string(&mut response).unwrap();
        assert_eq!("ERROR\r\n", response);
    }

    #[test]
    fn test_text_key_present() {
        let mut client_stream = make_server_conn();
        // If we ask for a key, we get its value.
        client_stream.write("get k".as_bytes()).unwrap();
        client_stream.shutdown(Shutdown::Write).unwrap();
        let mut response = String::new();
        client_stream.read_to_string(&mut response).unwrap();
        assert_eq!("VALUE k 0 1\r\nv\r\nEND\r\n", response);
    }

    #[test]
    fn test_text_key_absent() {
        let mut client_stream = make_server_conn();
        // If we ask for a key, we get its value.
        client_stream.write("get _".as_bytes()).unwrap();
        client_stream.shutdown(Shutdown::Write).unwrap();
        let mut response = String::new();
        client_stream.read_to_string(&mut response).unwrap();
        assert_eq!("END\r\n", response);
    }

    #[test]
    fn test_text_not_implemented() {
        let mut client_stream = make_server_conn();
        // If we send an unsuppoted command, we get an error.
        client_stream
            .write("set k 0 60 1\r\n_\r\n".as_bytes())
            .unwrap();
        client_stream.shutdown(Shutdown::Write).unwrap();
        let mut response = String::new();
        client_stream.read_to_string(&mut response).unwrap();
        assert_eq!(
            "SERVER_ERROR Read-only; method not implemented\r\n",
            response
        );
    }

    /*
    #[test]
    fn test_binary_key_present() {
        let mut client_stream = make_server_conn();
        client_stream
            .write_request(&Request {
                header: RequestHeader {
                    magic: constants::REQUEST_MAGIC,
                    opcode: constants::opcodes::GET,
                    key_length: 1,
                    extras_length: 0,
                    data_type: 0x00,
                    reserved: 0,
                    total_body_length: 0,
                    opaque: 0,
                    cas: 0,
                },
                extras: vec![],
                key: vec!['k' as u8],
            })
            .unwrap();
        let response = client_stream.read_response().unwrap();
        assert_eq!(
            AResponse {
                header: ResponseHeader {
                    magic: constants::RESPONSE_MAGIC,
                    opcode: constants::opcodes::GET,
                    key_length: 0,
                    extras_length: 4,
                    data_type: 0x00,
                    status: constants::response_status::NO_ERROR,
                    total_body_length: 5,
                    opaque: 0,
                    cas: 0,
                },
                extras: vec![0, 0, 0, 0],
                key: vec![],
                value: vec!['v' as u8],
            },
            response
        );
    }

    #[test]
    fn test_binary_key_absent() {
        let mut client_stream = make_server_conn();
        client_stream
            .write_request(&Request {
                header: RequestHeader {
                    magic: constants::REQUEST_MAGIC,
                    opcode: constants::opcodes::GET,
                    key_length: 1,
                    extras_length: 0,
                    data_type: 0x00,
                    reserved: 0,
                    total_body_length: 0,
                    opaque: 0,
                    cas: 0,
                },
                extras: vec![],
                key: vec!['_' as u8],
            })
            .unwrap();
        let response = client_stream.read_response().unwrap();
        assert_eq!(
            AResponse {
                header: ResponseHeader {
                    magic: constants::RESPONSE_MAGIC,
                    opcode: constants::opcodes::GET,
                    key_length: 0,
                    extras_length: 0,
                    data_type: 0x00,
                    status: constants::response_status::KEY_NOT_FOUND,
                    total_body_length: 0,
                    opaque: 0,
                    cas: 0,
                },
                extras: vec![],
                key: vec![],
                value: vec![],
            },
            response
        );
    }

    #[test]
    fn test_binary_not_implemented() {
        let mut client_stream = make_server_conn();
        client_stream
            .write_request(&Request {
                header: RequestHeader {
                    magic: constants::REQUEST_MAGIC,
                    opcode: 0xff, // unsupported opcode
                    key_length: 0,
                    extras_length: 0,
                    data_type: 0x00,
                    reserved: 0,
                    total_body_length: 0,
                    opaque: 0,
                    cas: 0,
                },
                extras: vec![],
                key: vec![],
            })
            .unwrap();
        let response = client_stream.read_response().unwrap();
        assert_eq!(
            AResponse {
                header: ResponseHeader {
                    magic: constants::RESPONSE_MAGIC,
                    opcode: 0xff, // unsupported opcode
                    key_length: 0,
                    extras_length: 0,
                    data_type: 0x00,
                    status: constants::response_status::NOT_SUPPORTED,
                    total_body_length: 0,
                    opaque: 0,
                    cas: 0,
                },
                extras: vec![],
                key: vec![],
                value: vec![],
            },
            response
        );
    }
    */
}
