//! This library lets you deserialize the output of [`Debug`] into a serde
//! struct.
//!
//! If you would like to do the opposite (output [`Debug`] via [`Serialize`])
//! take a look at [`serde_fmt`](https://docs.rs/serde_fmt).
//!
//! # Getting Started
//! Add `serde_dbgfmt` to your `Cargo.toml`:
//! ```toml
//! [dependencies]
#![doc = concat!(env!("CARGO_PKG_NAME"), " = \"", env!("CARGO_PKG_VERSION"), "\"")]
//! ```
//!
//! # Deserializing a struct
//! ```
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize)]
//! struct Test {
//!     message: String,
//! }
//!
//! let text = format!("{:?}", Test { message: "Hello, World!".into() });
//! let value: Test = serde_defmt::from_str(&text)
//!     .expect("failed to deserialize from the debug repr");
//!
//! assert_eq!(value.message, "Hello, World!");
//! ```
//!
//! # Caveats and Limitations
//! - This library parses the format emitted by the debug helpers in
//!   [`std::fmt`]. Custom debug representations will not necessarily use these
//!   debug helpers and may not emit output that is compatible with them.
//! - The debug format emitted by the helpers above is not guaranteed to be
//!   stable. While it has remained remarkably stable there is no guarantee
//!   that it will not be changed in the future.
//! - The names of the structs used to deserialize must match those in the text
//!   debug representation. You can use `#[serde(rename = "..")]` if you want to
//!   use a different struct name in your codebase.
//!
//! [`Debug`]: std::fmt::Debug
//! [`Serialize`]: serde::Serialize

use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Deserialize;

mod de;
mod error;
mod lex;

pub use crate::de::Deserializer as Deserializer;
pub use crate::error::Error;

/// Parse a `T` from the string containing its debug representation.
pub fn from_str<'de, T>(str: &'de str) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    let mut de = Deserializer::new(str);
    let value = T::deserialize(&mut de)?;
    de.end()?;
    Ok(value)
}

/// Parse the debug representation of `U` as a `T`.
pub fn from_dbg<T, U>(value: &U) -> Result<T, Error> 
where
    T: DeserializeOwned,
    U: Debug
{
    from_str(&format!("{value:?}"))
}
