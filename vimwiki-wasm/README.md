# vimwiki wasm

Provides wasm bindings for vimwiki library.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
vimwiki-server = "0.1.0-alpha.5"
```

## Examples

Embedding and running the server from your own binary:

```typescript
use v::{Program, Config};

#[tokio::main]
async fn main() {
    // Load configuration for the server from the CLI arguments
    let config = Config::load();

    // Start the server program
    Program::run(config).await.expect("Server failed unexpectedly");
}
```

## License

This project is licensed under either of

Apache License, Version 2.0, (LICENSE-APACHE or
[apache-license][apache-license]) MIT license (LICENSE-MIT or
[mit-license][mit-license]) at your option.

[apache-license]: http://www.apache.org/licenses/LICENSE-2.0
[mit-license]: http://opensource.org/licenses/MIT
