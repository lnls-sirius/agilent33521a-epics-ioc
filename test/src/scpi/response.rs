use bytes::BytesMut;

#[derive(Clone)]
pub enum ScpiResponse {
    Empty,
    Integer(isize),
    Utf8String(String),
}

impl ScpiResponse {
    pub fn encode(&self, buffer: &mut BytesMut) {
        match *self {
            ScpiResponse::Empty => {}
            ScpiResponse::Integer(value) => {
                buffer.extend(value.to_string().as_bytes())
            }
            ScpiResponse::Utf8String(ref string) => {
                buffer.extend(string.as_bytes())
            }
        }
    }
}
