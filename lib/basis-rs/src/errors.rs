error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors {
        BasisError(msg: String)
    }
}
