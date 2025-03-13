use std::io::Read;

use crate::utils::header::Header;

pub struct Res {
    pub code: u16,
    pub header: Header,
    pub body: Body,
}

pub enum Body {
    Empty,
    Text(String),
    Stream(Box<dyn Read + Send + Sync>),
}

