use std::io;

use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::Framed;
use tokio_proto::pipeline::ServerProto;

use line_codec::LineCodec;

pub struct LineProtocol {
    separator: u8,
}

impl LineProtocol {
    pub fn with_separator(separator: u8) -> Self {
        Self { separator }
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for LineProtocol {
    type Request = String;
    type Response = String;
    type Transport = Framed<T, LineCodec>;
    type BindTransport = io::Result<Self::Transport>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec::with_separator(self.separator)))
    }
}
