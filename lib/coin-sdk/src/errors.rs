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
        Binance(::binance::errors::Error, binance::errors::ErrorKind);
    }

    foreign_links {
        ParseError(::std::num::ParseFloatError);
        SerdeJsonError(::serde_json::error::Error);
        StdIo(::std::io::Error);
    }
}
