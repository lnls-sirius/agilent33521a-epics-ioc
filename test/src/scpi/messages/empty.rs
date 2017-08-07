use bytes::BytesMut;

use super::super::message::ScpiMessage;

pub struct EmptyMessage;

impl ScpiMessage for EmptyMessage {
    fn encode(&self, _: &mut BytesMut) {}
}
