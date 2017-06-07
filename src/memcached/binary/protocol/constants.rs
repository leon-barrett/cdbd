//! Constants taken from
//! https://code.google.com/p/memcached/wiki/BinaryProtocolRevamped

#![allow(dead_code)]

pub const REQUEST_MAGIC: u8 = 0x80;
pub const RESPONSE_MAGIC: u8 = 0x81;

pub const RAW_BYTES: u8 = 0x00;

/// Response status
pub mod response_status {
    pub const NO_ERROR: u16 = 0x0000;
    pub const KEY_NOT_FOUND: u16 = 0x0001;
    pub const KEY_EXISTS: u16 = 0x0002;
    pub const VALUE_TOO_LARGE: u16 = 0x0003;
    pub const INVALID_ARGUMENTS: u16 = 0x0004;
    pub const ITEM_NOT_STORED: u16 = 0x0005;
    pub const INCR_DECR_ON_NON_NUMERIC_VALUE: u16 = 0x0006;
    pub const THE_VBUCKET_BELONGS_TO_ANOTHER_SERVER: u16 = 0x0007;
    pub const AUTHENTICATION_ERROR: u16 = 0x0008;
    pub const AUTHENTICATION_CONTINUE: u16 = 0x0009;
    pub const UNKNOWN_COMMAND: u16 = 0x0081;
    pub const OUT_OF_MEMORY: u16 = 0x0082;
    pub const NOT_SUPPORTED: u16 = 0x0083;
    pub const INTERNAL_ERROR: u16 = 0x0084;
    pub const BUSY: u16 = 0x0085;
    pub const TEMPORARY_FAILURE: u16 = 0x0086;
}

/// Command opcodes
pub mod opcodes {
    pub const GET: u8 = 0x00;
    pub const SET: u8 = 0x01;
    pub const ADD: u8 = 0x02;
    pub const REPLACE: u8 = 0x03;
    pub const DELETE: u8 = 0x04;
    pub const INCREMENT: u8 = 0x05;
    pub const DECREMENT: u8 = 0x06;
    pub const QUIT: u8 = 0x07;
    pub const FLUSH: u8 = 0x08;
    pub const GETQ: u8 = 0x09;
    pub const NO_OP: u8 = 0x0a;
    pub const VERSION: u8 = 0x0b;
    pub const GETK: u8 = 0x0c;
    pub const GETKQ: u8 = 0x0d;
    pub const APPEND: u8 = 0x0e;
    pub const PREPEND: u8 = 0x0f;
    pub const STAT: u8 = 0x10;
    pub const SETQ: u8 = 0x11;
    pub const ADDQ: u8 = 0x12;
    pub const REPLACEQ: u8 = 0x13;
    pub const DELETEQ: u8 = 0x14;
    pub const INCREMENTQ: u8 = 0x15;
    pub const DECREMENTQ: u8 = 0x16;
    pub const QUITQ: u8 = 0x17;
    pub const FLUSHQ: u8 = 0x18;
    pub const APPENDQ: u8 = 0x19;
    pub const PREPENDQ: u8 = 0x1a;
    pub const VERBOSITY: u8 = 0x1b;
    pub const TOUCH: u8 = 0x1c;
    pub const GAT: u8 = 0x1d;
    pub const GATQ: u8 = 0x1e;
    pub const SASL_LIST_MECHS: u8 = 0x20;
    pub const SASL_AUTH: u8 = 0x21;
    pub const SASL_STEP: u8 = 0x22;
    pub const RGET: u8 = 0x30;
    pub const RSET: u8 = 0x31;
    pub const RSETQ: u8 = 0x32;
    pub const RAPPEND: u8 = 0x33;
    pub const RAPPENDQ: u8 = 0x34;
    pub const RPREPEND: u8 = 0x35;
    pub const RPREPENDQ: u8 = 0x36;
    pub const RDELETE: u8 = 0x37;
    pub const RDELETEQ: u8 = 0x38;
    pub const RINCR: u8 = 0x39;
    pub const RINCRQ: u8 = 0x3a;
    pub const RDECR: u8 = 0x3b;
    pub const RDECRQ: u8 = 0x3c;
    pub const SET_VBUCKET: u8 = 0x3d;
    pub const GET_VBUCKET: u8 = 0x3e;
    pub const DEL_VBUCKET: u8 = 0x3f;
    pub const TAP_CONNECT: u8 = 0x40;
    pub const TAP_MUTATION: u8 = 0x41;
    pub const TAP_DELETE: u8 = 0x42;
    pub const TAP_FLUSH: u8 = 0x43;
    pub const TAP_OPAQUE: u8 = 0x44;
    pub const TAP_VBUCKET_SET: u8 = 0x45;
    pub const TAP_CHECKPOINT_START: u8 = 0x46;
    pub const TAP_CHECKPOINT_END: u8 = 0x47;
}
