use std::io;
use std::str::Utf8Error;

error_chain! {
    foreign_links {
        Io(io::Error);
        InvalidScpiMessage(Utf8Error);
    }

    errors {
        UnknownScpiMessage(message: String) {
            description("unknown SCPI message")
            display("unknown SCPI message: {}", message)
        }
    }
}
