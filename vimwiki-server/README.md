# vimwiki server

Provides graphql server to inspect and manipulate vimwiki files.

While this was supposed to be named `vimwiki_server`, it accidentally was
published as `vimwiki-server` and now we're stuck with that name. :/

## Usage

### Binary

Download a binary and place in your path.

Or, install using cargo:

```bash
cargo install vimwiki-server
```

### Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
vimwiki-server = "0.1"
```

## Examples

### Binary

Running the binary from the command line with **http://127.0.0.1/graphiql**
enabled:

```bash
vimwiki-server --graphiql
```

### Library

Embedding and running the server from your own binary:

```rust
use vimwiki_server::{Config, Opt, Program};

#[tokio::main]
async fn main() {
    // Load configuration for the server from the CLI arguments
    let opt = Opt::load();

    // Read in a config file (or default config) for use by server
    let config = Config::load(&opt).unwrap();

    // Start the server program
    Program::run(opt, config)
        .await
        .expect("Server failed unexpectedly");
}
```

## License

This project is licensed under either of

Apache License, Version 2.0, (LICENSE-APACHE or
[apache-license][apache-license]) MIT license (LICENSE-MIT or
[mit-license][mit-license]) at your option.

[apache-license]: http://www.apache.org/licenses/LICENSE-2.0
[mit-license]: http://opensource.org/licenses/MIT
