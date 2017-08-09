use bytes::BytesMut;

#[derive(Clone)]
pub enum ScpiResponse {
    Empty,
}

impl ScpiResponse {
    pub fn encode(&self, _buffer: &mut BytesMut) {
        match *self {
            ScpiResponse::Empty => {},
        }
    }
}
