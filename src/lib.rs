/// Deserialize rust's debug
use serde::Deserialize;

mod de;
mod error;
mod lex;

pub use crate::de::DebugDeserializer as Deserializer;
pub use crate::error::Error;

pub fn from_str<'de, T>(str: &'de str) -> Result<T, Error<'de>>
where
    T: Deserialize<'de>,
{
    let mut de = Deserializer::new(str);
    let value = T::deserialize(&mut de)?;
    de.finish()?;
    Ok(value)
}
