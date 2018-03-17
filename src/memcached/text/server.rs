use std::io::{BufRead, Write};

use kvstore::KvStore;

use super::protocol::{Request, Response};
use super::super::error::Result;

pub fn handle_client<KV: KvStore, T: BufRead, U: Write>(
    kvstore: KV,
    mut ins: T,
    mut outs: U,
) -> Result<()> {
    trace!("memcached_text:connect");
    loop {
        match Request::parse(&mut ins) {
            Request::Quit => {
                break;
            }
            Request::Closed => {
                break;
            }
            Request::Error => {
                trace!("memcached_text:error");
                try!(Response::Error.write(&mut outs));
            }
            Request::Get { keys, cas } => {
                trace!("memcached_text:get {:?}", keys);
                for key in keys.iter() {
                    match kvstore.get(key.as_bytes()) {
                        Some(value) => {
                            try!(
                                Response::KeyValue {
                                    key: key,
                                    flags: 0,
                                    value: &value,
                                    cas: if cas { Some(0) } else { None },
                                }.write(&mut outs)
                            );
                        }
                        None => {}
                    }
                }
                try!(Response::End.write(&mut outs));
            }
            op @ _ => {
                trace!("memcached_text:not implemented method: {:?}", op);
                try!(Response::ServerError("Read-only; method not implemented").write(&mut outs));
            }
        }
        try!(outs.flush());
    }
    trace!("memcached_text:disconnect");
    Ok(())
}
