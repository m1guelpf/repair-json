# repair_json

> Repair incomplete JSON (e.g. from streaming APIs or AI models) so it can be parsed as it's received.

[![crates.io](https://img.shields.io/crates/v/repair-json.svg)](https://crates.io/crates/repair_json)
[![download count badge](https://img.shields.io/crates/d/repair-json.svg)](https://crates.io/crates/repair_json)
[![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/repair_json)

## Usage

```rust
let json_stream = json_source::stream().await?;

while let Some(incomplete_json) = json_stream.next().await {
    let valid_json = repair_json::repair(incomplete_json);
    let parsed_struct = serde_json::from_str(valid_json).unwrap();

    // ...
}
```

Refer to the [documentation on docs.rs](https://docs.rs/repair_json) for detailed usage instructions.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
