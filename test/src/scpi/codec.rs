use std::str;

use bytes::BytesMut;
use tokio_io::codec::{Decoder, Encoder};

use super::errors::{Error, Result};
use super::message::ScpiMessage;
use super::requests::ScpiRequest;

pub struct ScpiCodec;

impl Encoder for ScpiCodec {
    type Item = Box<ScpiMessage>;
    type Error = Error;

    fn encode(
        &mut self,
        message: Self::Item,
        buffer: &mut BytesMut,
    ) -> Result<()> {
        message.encode(buffer);

        Ok(())
    }
}

impl Decoder for ScpiCodec {
    type Item = ScpiRequest;
    type Error = Error;

    fn decode(&mut self, buffer: &mut BytesMut) -> Result<Option<Self::Item>> {
        let message_end = buffer.iter().position(is_end_of_message);

        if let Some(message_end) = message_end {
            let message_bytes = buffer.split_to(message_end);
            let message = str::from_utf8(&message_bytes)?;

            buffer.split_to(1);

            Ok(Some(ScpiRequest::from(message)?))
        } else {
            Ok(None)
        }
    }
}

fn is_end_of_message(byte: &u8) -> bool {
    *byte == '\r' as u8 || *byte == '\n' as u8 || *byte == ';' as u8
}
