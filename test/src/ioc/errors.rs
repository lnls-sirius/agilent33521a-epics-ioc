use std::io;

error_chain! {
    foreign_links {
        Io(io::Error);
    }

    errors {
        IocStdinAccessError {
            description("failed to access child IOC process standard input")
        }

        IocWriteError {
            description("failed to write to child IOC process standard output")
        }

        SettingIocVariable {
            description("concurrent access while setting IOC variable")
        }

        IocInstancePolledAfterEnd {
            description("IOC instance Future was polled after it ended")
        }

        IocProcessPolledAfterEnd {
            description("IOC process Future was polled after it ended")
        }

        IocProcessPolledWhileCheckingForError {
            description("IOC process Future was polled while checking for \
                         error")
        }
    }
}
