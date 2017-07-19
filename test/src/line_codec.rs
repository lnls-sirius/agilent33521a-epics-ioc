use std::io;
use std::str;

use bytes::BytesMut;
use tokio_io::codec::{Decoder, Encoder};

fn invalid_utf8_error() -> io::Error {
    io::Error::new(io::ErrorKind::Other, "invalid UTF-8 message")
}

pub struct LineCodec {
    separator: u8,
}

impl LineCodec {
    pub fn with_separator(separator: u8) -> Self {
        Self { separator }
    }
}

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, buffer: &mut BytesMut) -> io::Result<Option<String>> {
        let separator = self.separator;
        let separator_pos = buffer.iter().position(|&byte| byte == separator);

        if let Some(separator_pos) = separator_pos {
            let line = buffer.split_to(separator_pos);

            buffer.split_to(1);

            str::from_utf8(&line)
                .map(|chars| Some(chars.to_string()))
                .map_err(|_| invalid_utf8_error())
        } else {
            Ok(None)
        }
    }
}

impl Encoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn encode(&mut self, msg: String, buf: &mut BytesMut) -> io::Result<()> {
        buf.extend(msg.as_bytes());
        buf.extend(&[self.separator]);

        Ok(())
    }
}
