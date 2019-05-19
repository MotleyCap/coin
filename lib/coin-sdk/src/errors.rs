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

    links {
        Coinbase(::coinbase::errors::Error, coinbase::errors::ErrorKind);
    }

    foreign_links {
        ParseError(::std::num::ParseFloatError);
    }
}
