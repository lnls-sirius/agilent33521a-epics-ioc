use bytes::BytesMut;

#[derive(Clone)]
pub enum ScpiResponse {
    Empty,
    Integer(isize),
}

impl ScpiResponse {
    pub fn encode(&self, buffer: &mut BytesMut) {
        match *self {
            ScpiResponse::Empty => {}
            ScpiResponse::Integer(value) => {
                buffer.extend(value.to_string().as_bytes())
            }
        }
    }
}
