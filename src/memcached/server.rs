use std::io::{BufReader, BufWriter, Cursor, Read};
use std::net::{TcpListener, TcpStream};
use std::thread;

use byteorder::ReadBytesExt;

use kvstore::KvStore;
use super::binary::protocol::constants as binary_constants;
use super::binary::server as binary_server;
use super::error::Result;
use super::text::server as text_server;

pub fn memcached_server<KV>(kvstore: KV, host: &str, port: u16)
where
    KV: KvStore,
    KV: Clone,
    KV: Send,
    KV: 'static,
{
    let listener = TcpListener::bind((host, port)).expect(&format!("Failed to open port {}", port));

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // connection succeeded
                let kvs = kvstore.clone();
                thread::spawn(move || handle_client(kvs, stream));
            }
            Err(_) => {
                trace!("connection failed as it was received");
            }
        }
    }
}

fn handle_client<KV: KvStore>(kvstore: KV, mut stream: TcpStream) -> Result<()> {
    let first_char: u8 = try!(stream.read_u8());
    let fake_stream = Cursor::new(vec![first_char]);
    let addr = try!(stream.peer_addr());
    let peeked = BufReader::new(fake_stream.chain(stream.try_clone().unwrap()));
    let writer = BufWriter::new(stream);
    let binary = first_char == binary_constants::REQUEST_MAGIC;
    let protocol_name = match binary {
        true => "memcached_binary",
        false => "memcached_text",
    };
    info!("{} connection from {}", protocol_name, addr);
    let result = match binary {
        true => binary_server::handle_client(kvstore, peeked, writer),
        _ => text_server::handle_client(kvstore, peeked, writer),
    };
    info!("{} disconnection from {}", protocol_name, addr);
    result
}

#[cfg(test)]
mod test {
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpListener, TcpStream};
    use std::thread;

    use kvstore::KvStore;
    use super::super::binary::protocol::{constants, AResponse, PRead, PWrite, Request,
                                         RequestHeader, ResponseHeader};

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

    fn make_server_conn() -> TcpStream {
        let listener = TcpListener::bind(("localhost", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let client_conn = TcpStream::connect(("localhost", port)).unwrap();
        thread::spawn(move || {
            let (server_stream, _) = listener.accept().unwrap();
            super::handle_client(DummyKvStore {}, server_stream).unwrap_or(());
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
}
