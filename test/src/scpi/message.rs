use bytes::BytesMut;

pub trait ScpiMessage {
    fn encode(&self, buffer: &mut BytesMut);
}
