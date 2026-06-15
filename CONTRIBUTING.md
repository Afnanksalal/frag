# Contributing

Thanks for your interest in Frag.

Frag is an educational compiler, so clarity matters as much as correctness. Prefer readable code and focused changes over clever abstractions.

## Development Setup

Required:

```bash
rustup toolchain install 1.96.0 --component clippy --component rustfmt
cargo build
```

Recommended external tools:

```bash
sudo apt-get install -y iverilog verilator graphviz gtkwave
```

MSYS2 MinGW64 users can install the same tools with:

```bash
pacman -S --needed mingw-w64-x86_64-iverilog mingw-w64-x86_64-verilator mingw-w64-x86_64-graphviz mingw-w64-x86_64-gtkwave
```

## Checks Before A Pull Request

Rust-only checks:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps
cargo build --release
```

External-tool checks:

```bash
FRAG_REQUIRE_EXTERNAL_TOOLS=1 cargo test
```

## Coding Guidelines

- Keep the compiler pipeline explicit: lexer, parser, AST, semantic analysis, IR, backend.
- Do not generate Verilog directly from the AST.
- Keep diagnostics actionable and include source spans when possible.
- Add tests for parser, semantic, IR, simulator, and backend behavior when touching those areas.
- Prefer small functions with clear names over broad abstractions.
- Add comments for non-obvious compiler or hardware reasoning, not for code that already reads clearly.

## Pull Request Guidelines

Please include:

- What changed
- Why it changed
- How it was tested
- Any follow-up work you intentionally left out

Small PRs are easier to review.
