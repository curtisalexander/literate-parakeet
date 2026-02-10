# gather

Fast context gathering for AI coding agents.

`gather` walks a codebase, respects `.gitignore`, filters by glob patterns, and outputs file contents in structured formats (Markdown or XML) ready to paste into an AI context window. It also estimates token counts.

Built in Rust, distributed as a Python package via [maturin](https://github.com/PyO3/maturin).

## Install

Precompiled wheels are attached to each [GitHub Release](https://github.com/curtisalexander/literate-parakeet/releases). Install directly with `uv` — no Rust toolchain required:

```sh
# Install as a standalone CLI tool (recommended — adds `gather` to your PATH)
uv tool install gather --find-links https://github.com/curtisalexander/literate-parakeet/releases/expanded_assets/v0.1.0

# Or run directly without installing
uvx --from gather --find-links https://github.com/curtisalexander/literate-parakeet/releases/expanded_assets/v0.1.0 gather collect .

# Or install into the current environment
uv pip install gather --find-links https://github.com/curtisalexander/literate-parakeet/releases/expanded_assets/v0.1.0
```

To upgrade, pass `--upgrade` (or `--reinstall` for the same version):

```sh
uv tool install --upgrade gather --find-links https://github.com/curtisalexander/literate-parakeet/releases/expanded_assets/v0.2.0
```

To uninstall:

```sh
uv tool uninstall gather
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
pyproject.toml          # Python/maturin build config (bindings = "bin")
src/main.rs             # Rust CLI implementation
.github/workflows/
  ci.yml                # CI: test + build wheels + release
```

The Rust binary is compiled by [maturin](https://www.maturin.rs/) with `bindings = "bin"`. This places the compiled binary directly into the wheel's `data/scripts/` directory. When installed via `pip` or `uv`, the binary lands in the environment's `bin/` directory — no Python wrapper needed.

## Development

### Rust-only (no Python packaging)

```sh
cargo build
cargo run -- collect . --tokens
cargo test
```

### Full build with uv + maturin

```sh
# Create a virtual environment and install the maturin build tool
uv venv
uv pip install maturin

# Build the Rust binary and install it into the virtual environment
uv run maturin develop

# Now `gather` is available inside the venv
uv run gather collect . --tokens
```
