# Nemesis

TODO:

- Consider ArgosCI integration
- Consider needed dependencies in `Cargo.toml`

The error handling crate for my ecosystem

It follows my "All code written by me or part of rust's standard library and libc" philosophy.
You can learn more about that [here](https://blog.xqhare.net/posts/why_solve_problems/).

## Features

- _**No dependencies**_: All code is written by me or part of std.

## Environment

Nemesis expects the environment to provide:

- `ls` UNIX command

## Naming

As with all my projects, Nemesis is named after an ancient deity.
Learn more about my naming scheme [here](https://blog.xqhare.net/posts/explaining_the_pantheon/).

In ancient Greek religion and myth, Nemesis was the goddess who personified retribution for the sin of hubris: arrogance before the gods.

## Usage

### Importing

Add the following to your `Cargo.toml`:

```toml
[dependencies]
Nemesis = { git = "https://github.com/xqhare/nemesis" }
```

### Example

```rust

```

## License

Nemesis is distributed under the [MIT](https://github.com/xqhare/nemesis/blob/master/LICENSE) license.

## Contributing

See [CONTRIBUTING](https://github.com/xqhare/nemesis/blob/master/CONTRIBUTING.md) for contribution guidelines.
