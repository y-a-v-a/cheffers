# Chef Interpreter

This project implements a partial interpreter for the [Chef programming language](https://www.dangermouse.net/esoteric/chef.html) in Rust. Chef is an esoteric language where programs are written to resemble cooking recipes. Ingredients represent variables, mixing bowls are stacks, and instructions such as “Put sugar into the mixing bowl” or “Pour contents of the mixing bowl into the baking dish” map to mutations of program state. When everything succeeds, the finished “dish” doubles as the program’s output.

## Project Overview

- `src/lib.rs` exposes reusable modules for parsing Chef recipes (`parser`), interpreting the resulting instruction stream (`interpreter`), the instruction enum itself (`instruction`), and shared type definitions (`types`).
- `src/main.rs` contains the binary entry point. It wires command-line input to the parser and interpreter.
- `tests/recipes.rs` provides integration tests that exercise real Chef recipes stored under `tests/fixtures/`.

Current interpreter support covers a subset of Chef instructions (put, fold, add, serve with auxiliary recipes, etc.). Loop constructs are parsed but not yet fully executed, which is why the Fibonacci fixture currently fails during parsing—this is codified in the tests to capture the existing limitation.

## Usage

### Running a Recipe

1. Place a `.chef` file somewhere accessible (you can use the supplied fixtures under `tests/fixtures/`).
2. Run the interpreter with Cargo, optionally passing the recipe path:

```bash
cargo run -- tests/fixtures/hello-world.chef
```

If no path is provided, the interpreter defaults to `hello.chef` in the project root.

### Running the Test Suite

```bash
cargo fmt
cargo clippy
cargo test
```

`cargo test` runs both the unit tests found alongside the source files and the integration tests in `tests/recipes.rs`.

## Learning More About Chef

The original language specification, examples, and additional background information are available at [dangermouse.net/esoteric/chef](https://www.dangermouse.net/esoteric/chef.html). It is an excellent starting point if you want to write new recipes to feed into this interpreter or improve the runtime to support more of the language.
