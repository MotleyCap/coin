use std;
use reqwest;
use serde_json;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors {
        CoinbaseError(code: i16, msg: String, response: reqwest::Response)
    }

    foreign_links {
        ReqError(reqwest::Error);
        InvalidHeaderError(reqwest::header::InvalidHeaderValue);
        IoError(std::io::Error);
        ParseFloatError(std::num::ParseFloatError);
        Json(serde_json::Error);
        TimestampError(std::time::SystemTimeError);
    }

}
