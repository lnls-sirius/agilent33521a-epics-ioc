use super::messages::empty::EmptyMessage;

pub struct Messages;

impl Messages {
    pub fn empty() -> EmptyMessage {
        EmptyMessage {}
    }
}
