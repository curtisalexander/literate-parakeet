# gather

Fast context gathering for AI coding agents.

`gather` walks a codebase, respects `.gitignore`, filters by glob patterns, and outputs file contents in structured formats (Markdown or XML) ready to paste into an AI context window. It also estimates token counts.

Built in Rust, distributed as a Python package via [maturin](https://github.com/PyO3/maturin).

## Install

```sh
# From the repo (via uv)
uv pip install git+https://github.com/curtisalexander/literate-parakeet.git

# Or run directly without installing
uvx --from git+https://github.com/curtisalexander/literate-parakeet.git gather collect .
```

## Usage

### Collect file contents

```sh
# Collect all files in the current directory as Markdown
gather collect .

# Filter to specific file types
gather collect . -g "*.rs" -g "*.toml"

# Exclude patterns
gather collect . -e "*.lock"

# Output as XML
gather collect . -f xml

# Show token count estimate in the output
gather collect . --tokens
```

### Tree view

```sh
# Show directory structure
gather tree .

# Filter the tree
gather tree . -g "*.py"
```

### Token estimation

```sh
# Estimate tokens per file
gather tokens .

# Only count Rust files
gather tokens . -g "*.rs"
```

## Architecture

```
Cargo.toml              # Rust project config
pyproject.toml          # Python/maturin build config
src/main.rs             # Rust CLI implementation
python/gather/          # Python wrapper (exec pattern)
  __init__.py           #   Binary locator + entry point
  __main__.py           #   python -m gather support
.github/workflows/
  ci.yml                # CI: test + build wheels
  release.yml           # Release: build + publish to PyPI
```

The Rust binary (`_gather`) is the core implementation. The Python package provides a thin wrapper that locates and `exec`s the binary, following the pattern described in [Distributing compiled binaries via Python](https://simonwillison.net/2026/Feb/4/distributing-go-binaries/).

## Development

```sh
# Build
cargo build

# Run directly
cargo run --bin _gather -- collect . --tokens

# Run tests
cargo test
pytest tests/
```
