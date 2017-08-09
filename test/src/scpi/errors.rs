use std::io;
use std::str::Utf8Error;

error_chain! {
    foreign_links {
        Io(io::Error);
        InvalidScpiMessage(Utf8Error);
    }

    errors {
        UnknownScpiRequest(message: String) {
            description("unknown SCPI request message")
            display("unknown SCPI request message: {}", message)
        }

        UnknownScpiMessage(message: String) {
            description("unknown SCPI message")
            display("unknown SCPI message: {}", message)
        }
    }
}
