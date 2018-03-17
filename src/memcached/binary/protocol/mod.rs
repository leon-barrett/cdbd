//! As described at
//! https://github.com/memcached/memcached/wiki/BinaryProtocolRevamped

use std::io::{Read, Result, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub mod constants;

/// A Memcached binary request header
#[derive(Debug, PartialEq, Eq)]
pub struct RequestHeader {
    pub magic: u8,
    pub opcode: u8,
    pub key_length: u16,
    pub extras_length: u8,
    pub data_type: u8,
    pub reserved: u16,
    pub total_body_length: u32,
    pub opaque: u32,
    pub cas: u64,
}

/// A Memcached binary request
#[derive(Debug, PartialEq, Eq)]
pub struct ARequest<T>
where
    T: AsRef<[u8]>,
{
    pub header: RequestHeader,
    pub extras: T,
    pub key: T,
}

pub type Request = ARequest<Vec<u8>>;

/// A Memcached binary response header
#[derive(Debug, PartialEq, Eq)]
pub struct ResponseHeader {
    pub magic: u8,
    pub opcode: u8,
    pub key_length: u16,
    pub extras_length: u8,
    pub data_type: u8,
    pub status: u16,
    pub total_body_length: u32,
    pub opaque: u32,
    pub cas: u64,
}

/// A Memcached binary response
#[derive(Debug, PartialEq, Eq)]
pub struct AResponse<T>
where
    T: AsRef<[u8]>,
{
    pub header: ResponseHeader,
    pub extras: T,
    pub key: T,
    pub value: T,
}

pub type Response<'a> = AResponse<&'a [u8]>;

pub trait PRead {
    fn read_request_header(&mut self) -> Result<RequestHeader>;
    fn read_request(&mut self) -> Result<Request>;
    fn read_response_header(&mut self) -> Result<ResponseHeader>;
    fn read_response(&mut self) -> Result<AResponse<Vec<u8>>>;
}

impl<T> PRead for T
where
    T: Read,
{
    fn read_request_header(self: &mut Self) -> Result<RequestHeader> {
        Ok(RequestHeader {
            magic: try!(self.read_u8()),
            opcode: try!(self.read_u8()),
            key_length: try!(self.read_u16::<BigEndian>()),
            extras_length: try!(self.read_u8()),
            data_type: try!(self.read_u8()),
            reserved: try!(self.read_u16::<BigEndian>()),
            total_body_length: try!(self.read_u32::<BigEndian>()),
            opaque: try!(self.read_u32::<BigEndian>()),
            cas: try!(self.read_u64::<BigEndian>()),
        })
    }

    fn read_request(self: &mut Self) -> Result<Request> {
        let header = try!(self.read_request_header());
        let (mut extras, mut key) = (Vec::new(), Vec::new());
        try!(
            self.take(header.extras_length as u64)
                .read_to_end(&mut extras)
        );
        try!(self.take(header.key_length as u64).read_to_end(&mut key));
        Ok(Request {
            header: header,
            extras: extras,
            key: key,
        })
    }

    fn read_response_header(self: &mut Self) -> Result<ResponseHeader> {
        Ok(ResponseHeader {
            magic: try!(self.read_u8()),
            opcode: try!(self.read_u8()),
            key_length: try!(self.read_u16::<BigEndian>()),
            extras_length: try!(self.read_u8()),
            data_type: try!(self.read_u8()),
            status: try!(self.read_u16::<BigEndian>()),
            total_body_length: try!(self.read_u32::<BigEndian>()),
            opaque: try!(self.read_u32::<BigEndian>()),
            cas: try!(self.read_u64::<BigEndian>()),
        })
    }

    fn read_response(self: &mut Self) -> Result<AResponse<Vec<u8>>> {
        let header = try!(self.read_response_header());
        let mut extras = vec![0; header.extras_length as usize];
        let mut key = vec![0; header.key_length as usize];
        let value_length: usize = header.total_body_length as usize - header.extras_length as usize
            - header.key_length as usize;
        let mut value = vec![0; value_length];
        try!(self.read_exact(extras.as_mut_slice()));
        try!(self.read_exact(key.as_mut_slice()));
        try!(self.read_exact(value.as_mut_slice()));
        Ok(AResponse {
            header: header,
            extras: extras,
            key: key,
            value: value,
        })
    }
}

pub trait PWrite {
    fn write_request_header(&mut self, header: &RequestHeader) -> Result<()>;
    fn write_request<T: AsRef<[u8]>>(&mut self, request: &ARequest<T>) -> Result<()>;
    fn write_response_header(&mut self, header: &ResponseHeader) -> Result<()>;
    fn write_response<T: AsRef<[u8]>>(&mut self, response: &AResponse<T>) -> Result<()>;
}

impl<W: Write> PWrite for W {
    fn write_request_header(&mut self, header: &RequestHeader) -> Result<()> {
        try!(self.write_u8(header.magic));
        try!(self.write_u8(header.opcode));
        try!(self.write_u16::<BigEndian>(header.key_length));
        try!(self.write_u8(header.extras_length));
        try!(self.write_u8(header.data_type));
        try!(self.write_u16::<BigEndian>(header.reserved));
        try!(self.write_u32::<BigEndian>(header.total_body_length));
        try!(self.write_u32::<BigEndian>(header.opaque));
        try!(self.write_u64::<BigEndian>(header.cas));
        Ok(())
    }

    fn write_request<T: AsRef<[u8]>>(&mut self, request: &ARequest<T>) -> Result<()> {
        try!(self.write_request_header(&request.header));
        try!(self.write(request.extras.as_ref()));
        try!(self.write(request.key.as_ref()));
        try!(self.flush());
        Ok(())
    }

    fn write_response_header(&mut self, header: &ResponseHeader) -> Result<()> {
        try!(self.write_u8(header.magic));
        try!(self.write_u8(header.opcode));
        try!(self.write_u16::<BigEndian>(header.key_length));
        try!(self.write_u8(header.extras_length));
        try!(self.write_u8(header.data_type));
        try!(self.write_u16::<BigEndian>(header.status));
        try!(self.write_u32::<BigEndian>(header.total_body_length));
        try!(self.write_u32::<BigEndian>(header.opaque));
        try!(self.write_u64::<BigEndian>(header.cas));
        Ok(())
    }

    fn write_response<T: AsRef<[u8]>>(&mut self, response: &AResponse<T>) -> Result<()> {
        try!(self.write_response_header(&response.header));
        try!(self.write(response.extras.as_ref()));
        try!(self.write(response.key.as_ref()));
        try!(self.write(response.value.as_ref()));
        try!(self.flush());
        Ok(())
    }
}

impl<'a> Response<'a> {
    /// Construct a key/value response.
    pub fn make(
        request: &'a Request,
        extras: &'a [u8],
        include_key: bool,
        value: &'a [u8],
    ) -> Response<'a> {
        let key_length = if include_key {
            request.header.key_length
        } else {
            0
        };
        let len = (extras.len() + key_length as usize + value.len()) as u32;
        Response {
            header: ResponseHeader {
                magic: constants::RESPONSE_MAGIC,
                opcode: request.header.opcode,
                extras_length: extras.len() as u8,
                data_type: constants::RAW_BYTES,
                status: 0,
                key_length: key_length,
                total_body_length: len,
                opaque: request.header.opaque,
                cas: request.header.cas,
            },
            extras: extras,
            key: if include_key { &request.key } else { &[] },
            value: value,
        }
    }

    /// Construct an error response.
    pub fn make_error(request: &Request, status_code: u16) -> Response<'a> {
        Response {
            header: ResponseHeader {
                magic: constants::RESPONSE_MAGIC,
                opcode: request.header.opcode,
                extras_length: 0,
                data_type: constants::RAW_BYTES,
                status: status_code,
                key_length: 0,
                total_body_length: 0,
                opaque: request.header.opaque,
                cas: request.header.cas,
            },
            extras: &[],
            key: &[],
            value: &[],
        }
    }
}
