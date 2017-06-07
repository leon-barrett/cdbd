use std::io;
use std::io::{BufRead, Write};
use super::super::error::{Error, Result};

/// A number of commands have the same arguments
#[derive(Debug)]
pub struct DataRequest {
    key: String,
    flags: u16,
    exptime: u64,
    value: Vec<u8>,
}

#[derive(Debug)]
pub struct IncrRequest {
    key: String,
    value: u64,
    noreply: bool,
}

// As defined at https://github.com/memcached/memcached/blob/master/doc/protocol.txt
#[derive(Debug)]
pub enum Request {
    Get {
        keys: Vec<String>,
        cas: bool,
    },
    Set(DataRequest),
    Add(DataRequest),
    Replace(DataRequest),
    Append(DataRequest),
    Prepend(DataRequest),
    Cas {
        data: DataRequest,
        cas: u64,
    },
    Delete {
        key: String,
        noreply: bool,
    },
    Incr(IncrRequest),
    Decr(IncrRequest),
    Touch {
        key: String,
        exptime: u64,
        noreply: bool,
    },
    Stats(String),
    FlushAll,
    Version,
    Quit,
    Slabs(String),
    Error,
    Closed,
}

#[allow(dead_code)]
pub enum Response<'a> {
    KeyValue {
        key: &'a str,
        flags: u16,
        value: &'a [u8],
        cas: Option<u64>,
    },
    End,
    Error,
    ClientError(&'a str),
    ServerError(&'a str),
    NotFound,
    Deleted,
    Touched,
    NoReply,
    Ok,
    Stats(&'a [(&'a str, &'a str)]),
}

fn get_keys(ks: &[&str]) -> Vec<String> {
    ks.iter().map(|k| k.to_string()).collect()
}

fn read_data_request(elts: &[&str], rdr: &mut BufRead) -> Result<DataRequest> {
    if elts.len() != 5 {
        return Err(Error::from("wrong number of args for data request"));
    }
    let key = elts[1].to_string();
    let flags = try!(elts[2].parse());
    let exptime = try!(elts[3].parse());
    let length = try!(elts[4].parse());
    let mut value: Vec<u8> = vec![0; length];
    try!(rdr.read_exact(value.as_mut_slice()));
    let mut endline = vec![0; 2];
    // Read the endline characters
    try!(rdr.read_exact(endline.as_mut_slice()));
    if endline != "\r\n".as_bytes() {
        return Err(Error::from("missing newline at end of value"));
    }
    Ok(DataRequest {
        key: key,
        flags: flags,
        exptime: exptime,
        value: value,
    })
}

fn read_cas(elts: &[&str], rdr: &mut BufRead) -> Result<Request> {
    let key = elts[1].to_string();
    let flags = try!(elts[2].parse());
    let exptime = try!(elts[3].parse());
    let length = try!(elts[4].parse());
    let cas = try!(elts[5].parse());
    let mut value: Vec<u8> = vec![0; length];
    try!(rdr.read_exact(value.as_mut_slice()));
    Ok(Request::Cas {
        data: DataRequest {
            key: key,
            flags: flags,
            exptime: exptime,
            value: value,
        },
        cas: cas,
    })
}

fn parse_touch(elts: &[&str]) -> Result<Request> {
    Ok(Request::Touch {
        key: elts[1].to_string(),
        exptime: try!(elts[2].parse()),
        noreply: elts.get(3) == Some(&"noreply"),
    })
}

fn parse_incr(elts: &[&str]) -> Result<IncrRequest> {
    Ok(IncrRequest {
        key: elts[1].to_string(),
        value: try!(elts[2].parse()),
        noreply: elts.get(3) == Some(&"noreply"),
    })
}

impl Request {
    pub fn parse(rdr: &mut BufRead) -> Request {
        let mut cmd = String::new();
        // There's surely some tidier way to write this mass of conditional matches.
        match rdr.read_line(&mut cmd) {
            Err(_) => Request::Closed,
            Ok(0) => Request::Closed,
            Ok(_) => {
                let elts: Vec<&str> = cmd.split_whitespace().collect();
                match elts.len() {
                    0 => Request::Error,
                    _ => {
                        match (elts[0], elts.len()) {
                            ("get", _) => {
                                Ok(Request::Get {
                                    keys: get_keys(&elts[1..]),
                                    cas: false,
                                })
                            }
                            ("gets", _) => {
                                Ok(Request::Get {
                                    keys: get_keys(&elts[1..]),
                                    cas: true,
                                })
                            }
                            ("set", 5) => read_data_request(&elts, rdr).map(Request::Set),
                            ("add", 5) => read_data_request(&elts, rdr).map(Request::Add),
                            ("replace", 5) => read_data_request(&elts, rdr).map(Request::Replace),
                            ("append", 5) => read_data_request(&elts, rdr).map(Request::Append),
                            ("prepend", 5) => read_data_request(&elts, rdr).map(Request::Prepend),
                            ("cas", 6) => read_cas(&elts, rdr),
                            ("touch", 3...4) => parse_touch(&elts),
                            ("delete", 2...3) => {
                                Ok(Request::Delete {
                                    key: elts[1].to_string(),
                                    noreply: elts.get(2) == Some(&"noreply"),
                                })
                            }
                            ("incr", 3...4) => parse_incr(&elts).map(Request::Incr),
                            ("decr", 3...4) => parse_incr(&elts).map(Request::Decr),
                            ("slabs", _) => Ok(Request::Slabs(cmd.to_string())),
                            ("stats", _) => Ok(Request::Stats(cmd.to_string())),
                            ("flush_all", 1) => Ok(Request::FlushAll),
                            ("version", 1) => Ok(Request::Version),
                            ("quit", 1) => Ok(Request::Quit),
                            _ => Ok(Request::Error),
                        }
                        .unwrap_or(Request::Error)
                    }
                }
            }
        }
    }
}

impl<'a> Response<'a> {
    pub fn write(&self, wtr: &mut Write) -> Result<()> {
        match self {
            &Response::KeyValue { key, flags, value, cas } => {
                match cas {
                    None => write!(wtr, "VALUE {} {} {}\r\n", key, flags, value.len()),
                    Some(cas) => write!(wtr, "VALUE {} {} {} {}\r\n", key, flags, value.len(), cas),
                }
                .and_then(|_| wtr.write(value))
                .and_then(|_| write!(wtr, "\r\n"))
            }
            &Response::End => write!(wtr, "END\r\n"),
            &Response::Error => write!(wtr, "ERROR\r\n"),
            &Response::Deleted => write!(wtr, "DELETED\r\n"),
            &Response::NotFound => write!(wtr, "NOT_FOUND\r\n"),
            &Response::Touched => write!(wtr, "TOUCHED\r\n"),
            &Response::Ok => write!(wtr, "OK\r\n"),
            &Response::NoReply => Ok(()),
            &Response::ServerError(msg) => write!(wtr, "SERVER_ERROR {}\r\n", msg),
            &Response::ClientError(msg) => write!(wtr, "CLIENT_ERROR {}\r\n", msg),
            &Response::Stats(msgs) => {
                msgs.iter()
                    .map(|&(name, value)| write!(wtr, "STAT {} {}\r\n", name, value))
                    .collect::<io::Result<Vec<()>>>()
                    .map(|_| ())
            }
        }
        .map_err(Error::from)
    }
}
