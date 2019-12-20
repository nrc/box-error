#![feature(try_trait)]

use std::error::Error;
use std::fmt;
use std::mem;
use std::ops::Try;

#[derive(Debug, Clone)]
pub struct BoxResult<T, E: ?Sized>(Result<T, Box<E>>);

impl<T, E: ?Sized> BoxResult<T, E> {
    #[allow(non_snake_case)]
    pub fn Ok(v: T) -> BoxResult<T, E> {
        BoxResult(Ok(v))
    }

    pub fn from_boxed_err(e: Box<E>) -> BoxResult<T, E> {
        BoxResult(Err(e))
    }

    pub fn as_ref(&self) -> &Result<T, &E> {
        unsafe { mem::transmute(&self.0) }
    }

    // TODO name
    pub fn as_ref2(&self) -> Result<&T, &E> {
        match &self.0 {
            Ok(t) => Ok(t),
            Err(e) => Err(e),
        }
    }

    pub fn into_err<E2>(self) -> BoxResult<T, E2>
    where
        Box<E>: Into<Box<E2>>,
    {
        self.map_err(Into::into)
    }
}

impl<T, E: ?Sized> BoxResult<T, E> {
    pub fn map<F, U>(self, f: F) -> BoxResult<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self.0 {
            Ok(t) => BoxResult(Ok(f(t))),
            Err(e) => BoxResult(Err(e)),
        }
    }

    pub fn map_err<F, E2>(self, f: F) -> BoxResult<T, E2>
    where
        F: FnOnce(Box<E>) -> Box<E2>,
    {
        match self.0 {
            Ok(t) => BoxResult(Ok(t)),
            Err(e) => BoxResult(Err(f(e))),
        }
    }

    pub fn ok(self) -> Option<T> {
        match self.0 {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }

    pub fn err(self) -> Option<Box<E>> {
        match self.0 {
            Ok(_) => None,
            Err(e) => Some(e),
        }
    }
}

impl<T, E: ?Sized + fmt::Debug> BoxResult<T, E> {
    pub fn unwrap(self) -> T {
        self.0.unwrap()
    }
}

impl<T: fmt::Debug, E: ?Sized> BoxResult<T, E> {
    pub fn unwrap_err(self) -> Box<E> {
        self.0.unwrap_err()
    }
}

impl<T, E> BoxResult<T, E> {
    #[allow(non_snake_case)]
    pub fn Err(v: E) -> BoxResult<T, E> {
        BoxResult(Err(Box::new(v)))
    }

    pub fn unbox(self) -> Result<T, E> {
        match self.0 {
            Ok(v) => Ok(v),
            Err(b) => Err(*b),
        }
    }
}

impl<T, E> From<Result<T, E>> for BoxResult<T, E> {
    fn from(r: Result<T, E>) -> BoxResult<T, E> {
        match r {
            Ok(v) => BoxResult(Ok(v)),
            Err(v) => BoxResult(Err(Box::new(v))),
        }
    }
}

impl<T, E: ?Sized> From<Result<T, Box<E>>> for BoxResult<T, E> {
    fn from(r: Result<T, Box<E>>) -> BoxResult<T, E> {
        BoxResult(r)
    }
}

impl<T, E> Try for BoxResult<T, E> {
    type Ok = T;
    type Error = E;

    fn into_result(self) -> Result<T, E> {
        match self.0 {
            Ok(v) => Ok(v),
            Err(e) => Err(*e),
        }
    }

    fn from_error(v: E) -> Self {
        BoxResult(Err(Box::new(v)))
    }

    fn from_ok(v: T) -> Self {
        Self::Ok(v)
    }
}

#[derive(Debug)]
pub struct AnyResult<T>(Result<T, AnyError>);
#[derive(Debug)]
pub struct AnyError(anyhow::Error);

impl<T> Try for AnyResult<T> {
    type Ok = T;
    type Error = AnyError;

    fn into_result(self) -> Result<T, AnyError> {
        self.0
    }

    fn from_error(e: AnyError) -> Self {
        AnyResult(Err(e))
    }

    fn from_ok(v: T) -> Self {
        AnyResult(Ok(v))
    }
}

impl<T, E: Error + Send + Sync + 'static> From<BoxResult<T, E>> for AnyResult<T> {
    fn from(r: BoxResult<T, E>) -> AnyResult<T> {
        match r.0 {
            Ok(v) => AnyResult(Ok(v)),
            Err(v) => AnyResult(Err(AnyError(v.into()))),
        }
    }
}

pub trait Downcast: Sized + Send + Sync + 'static + fmt::Debug + fmt::Display {
    fn other(r: AnyError) -> Self;

    fn cast(r: AnyError) -> Self {
        match r.0.downcast::<Self>() {
            Ok(e) => e,
            Err(r) => Self::other(AnyError(r)),
        }
    }
}

impl<T> AnyResult<T> {
    #[allow(non_snake_case)]
    pub fn Ok(v: T) -> AnyResult<T> {
        AnyResult(Ok(v))
    }

    #[allow(non_snake_case)]
    pub fn Err<E: Into<anyhow::Error>>(e: E) -> AnyResult<T> {
        AnyResult(Err(AnyError(e.into())))
    }

    /// Try to downcast to a concrete error type `E`.
    pub fn try_cast<E: Error + Send + Sync + 'static + fmt::Debug + fmt::Display>(
        self,
    ) -> Result<BoxResult<T, E>, AnyResult<T>> {
        match self.0 {
            Ok(v) => Ok(BoxResult::Ok(v)),
            Err(AnyError(e)) => match e.downcast::<Box<E>>() {
                Ok(v) => Ok(BoxResult::from_boxed_err(v)),
                Err(r) => Err(AnyResult(Err(AnyError(r)))),
            },
        }
    }

    pub fn cast<E: Downcast + Error + Send + Sync + 'static + fmt::Debug + fmt::Display>(
        self,
    ) -> BoxResult<T, E> {
        match self.0 {
            Ok(v) => BoxResult(Ok(v)),
            Err(e) => BoxResult(Err(Box::new(E::cast(e)))),
        }
    }

    /// Downcast, panics if the concrete type is not `E`.
    pub fn expect<E: Error + Send + Sync + 'static + fmt::Debug + fmt::Display>(
        self,
    ) -> BoxResult<T, E> {
        match self.try_cast() {
            Ok(r) => r,
            Err(AnyResult(Err(e))) => panic!("Found {:?}", e),
            Err(_) => panic!(),
        }
    }

    // For a `bail!`-like macro
    pub fn from_display<M>(message: M) -> AnyResult<T>
    where
        M: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        AnyResult(Err(AnyError(anyhow::Error::msg(message.to_string()))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum TestError {
        #[error("foo")]
        Foo,
        #[error("bar: {0}")]
        Bar(String),
        #[error("other")]
        Other(AnyError),
    }

    impl Downcast for TestError {
        fn other(r: AnyError) -> TestError {
            TestError::Other(r)
        }
    }

    impl From<String> for TestError {
        fn from(s: String) -> TestError {
            TestError::Bar(s)
        }
    }

    impl<T: Into<TestError>> From<T> for AnyError {
        fn from(e: T) -> AnyError {
            e.into().into()
        }
    }

    // Match from concrete error.
    fn mtch(r: BoxResult<i32, TestError>) {
        match r.as_ref() {
            Ok(_) => {}
            Err(TestError::Foo) => {}
            _ => {}
        }
        match r.as_ref2() {
            Ok(_) => {}
            Err(TestError::Foo) => {}
            _ => {}
        }
    }

    // Match from dynamic error.
    fn mtch_any(r: AnyResult<i32>) {
        let te = r.expect::<TestError>();
        match te.as_ref() {
            Ok(_) => {}
            Err(TestError::Foo) => {}
            _ => {}
        }
        match te.as_ref2() {
            Ok(_) => {}
            Err(TestError::Foo) => {}
            _ => {}
        }
    }

    // These examples show making it easy to convert from something like tipb error.
    // To concrete error.
    fn bar(r: Result<i32, String>) -> BoxResult<i32, TestError> {
        let r = r?;
        BoxResult::Ok(r)
    }

    // To dynamic error.
    fn bar_any(r: Result<i32, String>) -> AnyResult<i32> {
        let r = r?;
        AnyResult::Ok(r)
    }

    #[test]
    fn it_works() {
        let e = BoxResult::Err(TestError::Foo);
        mtch(e);
        let e = BoxResult::Err(TestError::Foo);
        mtch_any(e.into());
    }

    // TODO test combinator functions
}
