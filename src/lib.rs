//! Procure is a library for grabbing various types of metrics from a
//! Linux system.

// Externs
extern crate sysconf;
extern crate csv;
extern crate rustc_serialize;


// Imports
use std::io;
use std::result;
use std::num::ParseIntError;

// Exports
pub mod cpu;
pub mod process;

/// Custom Result type many `procure` methods return
pub type Result<T> = result::Result<T, Error>;

/// Custom Error type returned with `procure` [`Result`](type.Result.html)'s
#[derive(Debug)]
pub enum Error {
    RuntimeError(String),
    IoError(io::Error),
    ParseError(ParseIntError),
}