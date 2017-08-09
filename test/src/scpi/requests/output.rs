use super::ScpiRequest;

pub struct Builder {
    channel: usize,
}

impl Builder {
    pub fn with_channel(channel: usize) -> Builder {
        Builder { channel }
    }

    pub fn on(self) -> ScpiRequest {
        ScpiRequest::OutputOn(self.channel)
    }

    pub fn off(self) -> ScpiRequest {
        ScpiRequest::OutputOff(self.channel)
    }

    pub fn query(self) -> ScpiRequest {
        ScpiRequest::OutputStatus(self.channel)
    }
}
