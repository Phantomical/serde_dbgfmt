# serde_defmt

Deserialize the output of `Debug` into any type which implements `Deserialize`.

# Example
```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Test {
    message: String,
}

let text = format!("{:?}", Test { message: "Hello, World!".into() });
let value: Test = serde_defmt::from_str(&text)
    .expect("failed to deserialize from the debug repr");

assert_eq!(value.message, "Hello, World!");
```

# Caveats
- This library parses the format emitted by the debug helpers in `std::fmt`.
  This should cover all types with `#[derive(Debug)]` and many custom impls but
  since custom impls can do anything it is not guaranteed to work.
- The debug format emitted by the debug helpers is not guaranteed to be stable.
  While it has remained rather stable in the past there is no guarantee that it
  will not be changed in the future.
- The names of structs used to deserialize must match those in the text debug
  representation. You can use `#[serde(rename = "..")]` if you want to use a
  different struct name in your codebase.

# See Also
- The [`serde_fmt`] library is the inverse of this crate. It allows you to 
  print a debug representation for any struct which implements [`Serialize`].

[`serde_fmt`]: https://crates.io/crates/serde_fmt
