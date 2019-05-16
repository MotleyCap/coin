error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors {
        BasisError(msg: String) {
            description("BasisError")
            display("BasisError: '{}'", msg)
        }
    }
}
