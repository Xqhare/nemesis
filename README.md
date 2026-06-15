# Nemesis

The error handling crate for my ecosystem

It follows my "All code written by me or part of rust's standard library and libc" philosophy.
You can learn more about that [here](https://blog.xqhare.net/posts/why_solve_problems/).

## Features

- _**No dependencies**_: All code is written by me or part of std.

## Naming

As with all my projects, Nemesis is named after an ancient deity.
Learn more about my naming scheme [here](https://blog.xqhare.net/posts/explaining_the_pantheon/).

In ancient Greek religion and myth, Nemesis was the goddess who personified retribution for the sin of hubris: arrogance before the gods.

## Usage

### Importing

Add the following to your `Cargo.toml`:

```toml
[dependencies]
nemesis = { git = "https://github.com/xqhare/nemesis" }
```

### Example Usage

Here is a simple example showing how to nest errors, add contexts, print them, walk the error layers, and downcast to the leaf error:

```rust
use std::io;
use nemesis::{NemesisError, NemesisResultExt};

fn read_config(path: &str) -> Result<String, NemesisError> {
    std::fs::read_to_string(path).map_err(|err| {
        NemesisError::new("Origin", err).add_ctx(format!("Failed to read file: {path}"))
    })
}

fn load_config() -> Result<String, NemesisError> {
    read_config("config.xff")
        .add_source("Athena")
        .add_ctx("Loading subsystem config during startup")
}

fn main() {
    if let Err(err) = load_config() {
        // Print formatted nested error hierarchy
        eprintln!("{}", err);

        // Programmatic check: downcast to leaf standard error
        if let Some(io_err) = err.downcast_ref::<io::Error>() {
            if io_err.kind() == io::ErrorKind::NotFound {
                eprintln!("Configuration file not found.");
            }
        }
    }
}
```

For a complete overview of all APIs (including walking error chains and using `NemesisCollection`), see the runnable example under [examples/usage.rs](examples/usage.rs).

## License

Nemesis is distributed under the [MIT](https://github.com/xqhare/nemesis/blob/master/LICENSE) license.

## Contributing

See [CONTRIBUTING](https://github.com/xqhare/nemesis/blob/master/CONTRIBUTING.md) for contribution guidelines.
