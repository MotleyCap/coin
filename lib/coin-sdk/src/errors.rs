error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors {
        CoinError(msg: String) {
            description("CoinError")
            display("CoinError: '{}'", msg)
        }
    }
}
