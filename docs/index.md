---
layout: default
title: Home
---

# Cheffers Documentation

Welcome to the **Cheffers** documentation! Cheffers is a complete Rust interpreter for the Chef esoteric programming language, where programs are written as cooking recipes.

## What is Chef?

Chef is an esoteric programming language created by David Morgan-Mar in 2002. Programs in Chef look like cooking recipes - ingredients represent variables, mixing bowls are stacks, and cooking instructions translate to program operations. When executed, your recipe doesn't just cook a dish - it computes!

## Features

- **100% Complete** - All 62 specification tests passing
- **Fast & Efficient** - Built with Rust for optimal performance
- **Easy to Use** - Simple command-line interface
- **Well Tested** - Comprehensive test suite with CI/CD

## Getting Started

### Installation

```bash
# From GitHub
cargo install --git https://github.com/y-a-v-a/cheffers

# From source
git clone https://github.com/y-a-v-a/cheffers.git
cd cheffers
cargo install --path .
```

### Quick Example

```bash
# Run a Chef recipe
cheffers path/to/recipe.chef

# Try the included examples
cheffers tests/fixtures/hello-world.chef
cheffers tests/fixtures/fibonacci.chef
```

## Documentation

### Guides

- **[Understanding Stack-Based Programming in Chef](stack-based-programming.md)** - Learn how Chef's stack-based paradigm works and how to think in stacks instead of variables. Perfect for developers coming from JavaScript, Python, or other imperative languages.
- **[Fibonacci Program - Spec Analysis](fibonacci-spec-analysis.md)** - Deep dive into the canonical Fibonacci implementation, clarifying advanced patterns like recursive auxiliary calls, Refrigerate as conditional exit, and other spec ambiguities.

### Reference Materials

- **[Chef Language Specification](https://github.com/y-a-v-a/cheffers/blob/main/language-spec/Chef.md)** - Complete formal specification of the Chef programming language
- **[Example Recipes](https://github.com/y-a-v-a/cheffers/tree/main/tests/fixtures)** - Browse working Chef programs including Hello World, Fibonacci, and more

## Project Information

- **GitHub Repository:** [y-a-v-a/cheffers](https://github.com/y-a-v-a/cheffers)
- **Version:** 0.2.0
- **License:** WTFPL (Do What The Fuck You Want To Public License)
- **Language:** Rust (2021 edition)

## Contributing

Found a bug or want to contribute? Visit our [GitHub repository](https://github.com/y-a-v-a/cheffers) to open an issue or submit a pull request.

## External Resources

- **[Original Chef Website](https://www.dangermouse.net/esoteric/chef.html)** - The official Chef language homepage by David Morgan-Mar
- **[Esolang Wiki: Chef](https://esolangs.org/wiki/Chef)** - Community wiki page with examples and discussion

---

**Ready to start cooking with code?** Check out our [Stack-Based Programming Guide](stack-based-programming.md) to learn the fundamentals!
