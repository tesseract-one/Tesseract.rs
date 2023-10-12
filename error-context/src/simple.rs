#[cfg(not(feature = "nostd"))]
use std::error::Error;

#[cfg(feature = "nostd")]
use core::error::Error;

use core::{
    result::Result,
    fmt::Debug,
};

pub trait ErrorContext
    where
        Self: Sized
{
    fn context_panicable<T>(fun: impl FnOnce() -> Result<T, Self>) -> T
    where
        Self: Debug
    {
        let terror: Result<T, Self> = fun();
        terror.expect("Unrecoverable error")
    }
}

impl<E> ErrorContext for E where E: Error {
}