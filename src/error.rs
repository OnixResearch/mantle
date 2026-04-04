use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("configuration error: {message}"))]
    Config { message: String },

    #[snafu(display("I/O error: {source}"))]
    Io { source: std::io::Error },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
