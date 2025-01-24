# prefer-dynamic

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/WilliamVenner/prefer-dynamic/ci.yml?branch=main&style=flat-square&logo=github&logoColor=white "GitHub Workflow Status")](https://github.com/WilliamVenner/prefer-dynamic/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/prefer-dynamic?logo=rust&style=flat-square "Crates.io")](https://crates.io/crates/prefer-dynamic)

Simple rust crate that copies the pre-compiled dynamic `std` library to your target directory.
This is a convenience intended for programs that make use of `dylib` crates, or get compiled
with `-Cprefer-dynamic`.

## Usage

Add to your `Cargo.toml`:

```toml
[target.'cfg(any(unix, windows))'.dependencies]
prefer-dynamic = "0.2"

[target.'cfg(any(unix, windows))'.dev-dependencies]
prefer-dynamic = "0.2"
```

Create or edit `.cargo/config.toml` in the manifest root of your project
(create the folder `.cargo` if it does not exists), and add:

```toml
[target.'cfg(any(unix, windows))']
rustflags = ["-Cprefer-dynamic=yes", "-Crpath"]
```

## License

This software is distributed under the terms of both the MIT license and the Apache License (Version 2.0).
All contributions must be dual licensed Apache2/MIT unless otherwise stated.
