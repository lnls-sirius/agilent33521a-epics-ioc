use std::io;

error_chain! {
    foreign_links {
        Io(io::Error);
    }

    errors {
        IocStdinAccessError {
            description("failed to access child IOC process standard input")
        }
    }
}
