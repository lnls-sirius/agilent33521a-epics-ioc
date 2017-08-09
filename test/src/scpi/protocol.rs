use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::Framed;
use tokio_proto::pipeline::ServerProto;

use super::codec::ScpiCodec;
use super::errors::{Error, Result};
use super::requests::ScpiRequest;
use super::response::ScpiResponse;

pub struct ScpiProtocol;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for ScpiProtocol {
    type Request = ScpiRequest;
    type Response = ScpiResponse;
    type Error = Error;
    type Transport = Framed<T, ScpiCodec>;
    type BindTransport = Result<Self::Transport>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(ScpiCodec))
    }
}
