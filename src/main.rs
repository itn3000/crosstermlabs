extern crate crossterm;
extern crate cursive;
extern crate scopeguard;
extern crate thiserror;
extern crate tokio;
extern crate tui;
extern crate futures;
extern crate unicode_width;

use std::sync::mpsc;

mod test1;
mod test2;
mod test3;
mod test4;
mod test5;
mod test6;
mod test7;
mod test8;

#[derive(thiserror::Error, Debug)]
pub enum ApplicationError {
    #[error("crossterm error:{0:?}")]
    Crossterm(#[from] crossterm::ErrorKind),
    #[error("io error:{0:?}")]
    Io(#[from] std::io::Error),
    #[error("channel send error:{0:?}")]
    ChannelSendError(#[from] mpsc::SendError<String>),
    #[error("tokio channel string send error:{0:?}")]
    TokioChannelStringError(#[from] tokio::sync::mpsc::error::SendError<String>),
    #[error("tokio channel bool send error:{0:?}")]
    TokioChannelBoolError(#[from] tokio::sync::mpsc::error::SendError<bool>),
}


fn main() -> Result<(), ApplicationError> {
    // test2()?;
    // test3()?;
    // test4()?;
    // test5()?;
    // test6()?;
    test7::test7()?;
    // test8::test8();
    Ok(())
}
