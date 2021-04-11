use std::result::*;


pub enum Error {
    ErrorGeneric,
}

pub trait SDR {
    fn init() -> Result<(), Error>;
    fn open() -> Result<(), Error>;
    fn run() -> Result<(), Error>;
    fn close() -> Result<(), Error>;
}


pub struct FileSDR {
    pub path: String
}

