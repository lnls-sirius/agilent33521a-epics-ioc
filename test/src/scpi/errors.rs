error_chain! {
    errors {
        UnknownScpiMessage(message: String) {
            description("unknown SCPI message")
            display("unknown SCPI message: {}", message)
        }
    }
}
