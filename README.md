# box-error

A library for error handling using boxed errors.

This crate is work in progress, it is not ready for production use.

If errors are large and passed around a lot, then it can be more efficient to do so by reference. That usually means using `Box` since errors are usually passed up the stack. This crate aims to make such boxed errors ergonomic to use.

Goals:

* Support concrete and dynamic types for errors and make converting between the two easy.
* Interoperate with `std::Error` and `std::Result`.
* Don't require users to match using box patterns.
* Make it easy to work with errors from dependencies which do not implement `std::Error`.

In contrast to most error handling libraries, box-error provides a result type (a replacement for `std::Result`) rather than an error type. The intention is for users to use another crate (e.g., [thiserror](https://github.com/dtolnay/thiserror)) to help write and implement errors. Our result type (`BoxResult`) always keeps its error value as a boxed reference.

For dynamic error handling, we use [anyhow](https://github.com/dtolnay/anyhow) for the implementation, so we can easily support its features like backtraces, chaining, and downcasting. However, we wrap `anyhow::Error` in a new type `AnyError` and also wrap `std::Result<T, AnyError>` as `AnyResult`. That does not box its errors because `AnyError` is internally implemented using `Box`. We make inter-converting between `BoxResult`, `AnyResult`, and `std::Result` as easy as possible.

## Example

This example will not work today, but I'm pretty sure its possible and should work soon.

```rust
use box_error::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
enum MyError {
    #[error("foo")]
    Foo,
    #[error("bar: {0}")]
    Bar(String),
    #[error("other")]
    Other(#[source] AnyError),
}

impl From<::std::num::TryFromIntError> for MyError {
    fn from(e: ::std::num::TryFromIntError) -> MyError {
        MyError::Other(e.into())
    }
}

pub fn exported() -> AnyResult<u32> {
    let i = internal()?;
    AnyResult::Ok(i.try_into()?)
}

fn internal() -> BoxResult<i64, MyError> {
    if ... {
        BoxResult::Ok(42)
    } else {
        BoxResult::Err(MyError::Foo)
    }
}

fn handle_bar_err(r: AnyResult<u32>) -> AnyResult<u32> {
    r.map_err(|e| if let MyError::Bar(s) = e.as_ref {
        println!("got a Bar: `{}`", s);
        AnyResult::Err(MyError::Foo)
    } else { e })
}
```

## Comparison to anyhow

We use anyhow to implement `AnyError`/`AnyResult`, so there are many similarities between the two libraries and we owe a massive debt of thanks to anyhow for the implementation and inspiration for box-error.

There are two main differences in the philosophy of anyhow and box-error:

* Anyhow provides a dynamic error type, but no concrete type. Box-error provides a dynamic error type and some parts of a concrete type.
* Box-error forces errors to be boxed whereas anyhow does not (they may be boxed, but are not forced to be). Because of this assumption, inter-conversion between boxed concrete and boxed dynamic errors is more ergonomic.

The use case for anyhow is applications, and for thiserror is library crates. The primary use case for box-error is large applications with crate-like modules and/or module-like crates where both dynamic and concrete errors have advantages in different places.

There are also a few minor improvements over anyhow - for example, being able to transparently match on boxed errors, easier handling of errors which don't implement `std::Error`, etc.

It is important to note that box-error is not yet production ready, but anyhow is. So if you're wandering which to use in production code, don't use box-error.


## Implementation notes

It is possible to use `BoxResult<T, std::Error>` to represent a dynamically typed box result. That representation makes for a cleaner implementation, however, it would require reimplementing most of the anyhow library and I prefer to reuse it.


