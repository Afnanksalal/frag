# Release Process

Frag releases are tag-driven.

## Versioning

Frag is currently pre-1.0. Use alpha tags until the HDL surface is more stable.

Recommended first release:

```text
v0.1.0-alpha.1
```

## Pre-Release Checklist

Run:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
FRAG_REQUIRE_EXTERNAL_TOOLS=1 cargo test
cargo doc --no-deps
cargo build --release
```

Check generated Verilog:

```bash
mkdir -p target/verify/verilog
for file in examples/*.frag; do
  name="$(basename "$file" .frag)"
  ./target/release/frag verilog "$file" -o "target/verify/verilog/$name.v"
  iverilog -g2012 -tnull "target/verify/verilog/$name.v"
  verilator --lint-only -Wno-DECLFILENAME "target/verify/verilog/$name.v"
done
```

## Create A Release

Commit all release changes, then tag:

```bash
git tag -a v0.1.0-alpha.1 -m "Frag v0.1.0-alpha.1"
git push origin main
git push origin v0.1.0-alpha.1
```

The GitHub Actions release workflow will build platform artifacts and create a draft GitHub Release.

Review the draft release, edit notes if needed, then publish it from GitHub.

## Artifacts

The release workflow uploads:

- Linux x86_64 archive
- macOS Apple Silicon archive
- Windows x86_64 archive
- SHA-256 checksum files
