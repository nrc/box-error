#![feature(try_trait)]

use std::fmt;
use std::mem;
use std::ops::Try;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct BoxResult<T, E: ?Sized>(Result<T, Box<E>>);
