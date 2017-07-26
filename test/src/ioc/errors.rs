use std::io;

error_chain! {
    foreign_links {
        Io(io::Error);
    }

    errors {
        IocSpawnCancelled {
            description("cancelled IOC process start-up")
        }

        IocStdinAccessError {
            description("failed to access child IOC process standard input")
        }

        IocProcessPolledAfterEnd {
            description("IOC process Future was polled after it ended")
        }
    }
}
