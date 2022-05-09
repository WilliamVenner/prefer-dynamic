# prefer-dynamic

Simple Rust crate that copies the `std` and `test` dynamic libraries into the target directory at build time.

This is a convenience intended for programs that make use of `dylib` crates or compiling with `-Cprefer-dynamic`.

# Usage

Add to your `Cargo.toml`

```toml
[dependencies]
prefer-dynamic = "0"

[dev-dependencies]
prefer-dynamic = { version = "0", features = ["link-test"] }
```