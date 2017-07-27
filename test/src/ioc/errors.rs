use std::io;

use super::ioc_process::IocProcess;

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

        KillingIoc {
            description("IOC was accessed while it was being killed")
        }

        KilledIoc(process: IocProcess) {
            description("IOC was successfully killed")
        }
    }
}
