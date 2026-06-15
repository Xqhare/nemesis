
I want a unified error type that can be used for the entire pantheon.

## Problems to solve

The way I currently handle errors looses a lot of context. I often just bubble it up directly, making it hard to debug.

I also cast many errors into new types so I can use `?` to handle them (e.g. casting an `AthenaError` into a `TalosError`. The resulting error doesnt have any additional context meaning that the error output can be: `IO error, unable to open file`. What file? Why do I want to open it? Config, logging?? Is it an error raised by talos or upstream somewhere?)

## What I want

Most important is programmatically interacting with the error.
Second is easy understanding of the printed output in `stderr`.

Ideally an error should present like this:

```bash
$ cargo run
$ Error: IO error.
$   Context: Loading config file during startup.
$   Source: Athena // <-- Optional
$       Error: IO error
$           Context: Unable to open file: path/to/file.xff\n\n{...} 
$           Source: Origin // Maybe do this if this is the source of the error? Would make source no longer optional, but a lot more obvious.
```

Meaning I need a `.add_ctx` method and a `.add_source` method.
Also an implementation of `Display`, maybe even `Debug`.

The above really is just a nested error.

### Idea

Also have a type for a vector of errors, to be able to collect errors together.

```bash
$ Error collection 'startup':
$   Error: IO error.
$       Description: Loading config file during startup.
$       Context: Passed file 'path/to/file.xff' to athena to load for startup config loading.
$       Source: Athena
$           Error: IO error
$               Context: Unable to open file: path/to/file.xff\n\n{...} 
$               Source: Origin
$   Error: Parsing error.
$       Description: Unable to parse 'a_string' into a valid 'usize'.
$       Context: Tried to populate field 'a_field' inside config struct during startup.
$       Source: This_library
```

## Solution

Instead of doing my knee jerk reaction of trying to build a generalised struct, I think defining a trait or type would be better.
Really just define the API for the error type.
