use serde::Deserialize;

mod error;
mod lex;
mod de;


pub use crate::error::Error;
pub use crate::de::DebugDeserializer;

pub fn from_str<'de, T>(str: &'de str) -> Result<T, Error<'de>>
where
    T: Deserialize<'de>
{
    let mut de = DebugDeserializer::new(str);
    let value = T::deserialize(&mut de)?;
    de.finish()?;
    Ok(value)
}
