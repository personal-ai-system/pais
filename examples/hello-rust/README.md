# examples/hello-rust

A PAIS skill plugin written in rust.

## Installation

```bash
pais plugin install --dev .
```

## Usage

```bash
pais run examples/hello-rust greet
pais run examples/hello-rust greet Alice
pais run examples/hello-rust version
```

## Development

```bash
cargo build --release
```

## License

MIT
