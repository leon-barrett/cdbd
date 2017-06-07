use std::io::{Read, Write};

use kvstore::KvStore;

use super::protocol::{PRead, PWrite, Response};
use super::protocol::constants::{opcodes, response_status};
use super::super::error::Result;

pub fn handle_client<KV: KvStore, T: Read + PRead, U: Write>(kvstore: KV,
                                                             mut ins: T,
                                                             mut outs: U)
                                                             -> Result<()> {
    trace!("memcached_binary:connect");
    loop {
        let request = try!(ins.read_request());
        let opcode = request.header.opcode;
        match opcode {
            opcodes::GET | opcodes::GETQ | opcodes::GETK | opcodes::GETKQ => {
                let include_key = opcode == opcodes::GETK || opcode == opcodes::GETKQ;
                let return_not_found = opcode == opcodes::GET || opcode == opcodes::GETK;
                match kvstore.get(&request.key) {
                    Some(data) => {
                        trace!("memcached_binary:get {:?} => {} bytes",
                               request.key,
                               data.len());
                        try!(outs.write_response(&Response::make(&request,
                                                                 &[0x00, 0x00, 0x00, 0x00],
                                                                 include_key,
                                                                 &data)));
                    }
                    None => {
                        trace!("memcached_binary:get {:?} => not found", request.key);
                        if return_not_found {
                            try!(outs.write_response(
                                    &Response::make_error(&request,
                                                          response_status::KEY_NOT_FOUND)));
                        }
                    }
                }
            }
            opcodes::QUIT => {
                trace!("memcached_binary:quit");
                break;
            }
            opcodes::NO_OP => {
                trace!("memcached_binary:noop");
                try!(outs.write_response(&Response::make(&request, &[], false, &[])));
            }
            opcodes::VERSION => {
                trace!("memcached_binary:version");
                try!(outs.write_response(&Response::make(&request,
                                                         &[],
                                                         false,
                                                         "0.0.0".as_bytes())));
            }
            _ => {
                trace!("memcached_binary:unknown opcode {}", request.header.opcode);
                try!(outs.write_response(&Response::make_error(&request,
                                                               response_status::NOT_SUPPORTED)));
            }
        }
        try!(outs.flush());
    }
    Ok(())
}
