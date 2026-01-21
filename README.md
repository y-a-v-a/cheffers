# Cheffers - Chef Interpreter

A complete interpreter for the [Chef programming language](https://www.dangermouse.net/esoteric/chef.html) written in Rust. Chef is an esoteric language where programs are written to resemble cooking recipes. Ingredients represent variables, mixing bowls are stacks, and instructions such as "Put sugar into the mixing bowl" or "Pour contents of the mixing bowl into the baking dish" map to mutations of program state. When everything succeeds, the finished "dish" doubles as the program's output.

## Features

- **Complete Chef language support** - All language constructs including loops, auxiliary recipes, and metadata parsing
- **62/62 specification tests passing** - Fully compliant with the Chef language specification
- **Rich error messages** - Rust-style error output with helpful suggestions and language spec references
- **Fast and efficient** - Built with Rust for optimal performance
- **Easy to use CLI** - Simple command-line interface for running Chef recipes
- **Development environment** - Tmux-based split-pane workflow for rapid iteration

## Installation

### From Source

```bash
git clone https://github.com/y-a-v-a/cheffers.git
cd cheffers
cargo install --path .
```

### From GitHub (requires Rust/Cargo)

```bash
cargo install --git https://github.com/y-a-v-a/cheffers
```

## Usage

### Running a Recipe

Once installed, run any Chef recipe file:

```bash
cheffers path/to/recipe.chef
```

Try it with the included example recipes:

```bash
cheffers tests/fixtures/hello-world.chef
cheffers tests/fixtures/fibonacci.chef
```

If no path is provided, the interpreter defaults to `hello.chef` in the current directory.

### Development Environment (Tmux)

For an interactive development experience with instant feedback, use the tmux-based development environment:

```bash
# Start the development environment
./scripts/chef-dev.sh myrecipe.chef

# Or let it create a template for you
./scripts/chef-dev.sh
```

This creates a split-pane interface:
- **Left pane:** Text editor (uses `$EDITOR`, defaults to vim)
- **Right pane:** Auto-runner that watches your file and shows output on save

**Features:**
- Instant feedback - save your file to see results
- Rich error messages with helpful suggestions
- No need to manually rerun the interpreter
- Uses your preferred editor

See `scripts/README.md` for detailed usage instructions, keybindings, and tips.

**Prerequisites:** `tmux` (required), `entr` (optional, for better performance)

### Development Usage

For development, you can also run recipes directly with Cargo:

```bash
cargo run --release -- tests/fixtures/hello-world.chef
```

### Running the Test Suite

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run all tests (unit + integration)
cargo test

# Run just the spec tests
cargo test --test spec_fixtures
```

The test suite includes 62 specification tests that validate compliance with the Chef language specification.

## Project Structure

- `src/lib.rs` - Library entry point exposing the parser, interpreter, instructions, and types
- `src/main.rs` - CLI binary implementation
- `src/parser.rs` - Chef recipe parser
- `src/interpreter.rs` - Chef instruction interpreter
- `src/instruction.rs` - Instruction enum definitions
- `src/types.rs` - Shared type definitions
- `src/error_formatter.rs` - Rich error message formatting
- `src/error_context.rs` - Error context and language spec references
- `tests/spec_fixtures.rs` - 62 specification compliance tests
- `tests/recipes.rs` - Integration tests for example recipes
- `tests/fixtures/` - Example Chef recipes
- `tests/errors/` - Error test fixtures
- `scripts/chef-dev.sh` - Tmux-based development environment
- `scripts/README.md` - Development environment documentation

## Continuous Integration

The project uses GitHub Actions to automatically:
- Check code formatting with `rustfmt`
- Run `clippy` linting (treating warnings as errors)
- Build the project
- Run the complete test suite

All checks run on every push and pull request.

## License

This project is licensed under the WTFPL (Do What The Fuck You Want To Public License). See the `LICENSE` file for details.

## Learning More About Chef

The original language specification, examples, and additional background information are available at [dangermouse.net/esoteric/chef](https://www.dangermouse.net/esoteric/chef.html). The specification document is also included in this repository at `language-spec/Chef.md`.
