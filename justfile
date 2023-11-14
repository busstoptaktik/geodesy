# Justfile for the Rust Geodesy project.

alias l := list
alias r := run
alias t := test
alias tt := test-all
alias rr := run-all

# Harmless default
default: list

# list all justfile targets
list:
    just -l

# Basic test: Just library unit tests. Use target "test-all", "check" or "clean-check" for successively more in depth check ups.
test:
    cargo test --lib

# Unit tests, doc tests, and compiling of examples
test-all:
    cargo test --all

# Check that all tests pass, and that formatting and coding conventions are OK.
check:
    cargo clippy -- --deny warnings
    cargo clippy --tests -- --deny warnings
    cargo fmt -- --check
    cargo test
    cargo doc --all-features --no-deps
    cargo package --allow-dirty
    git status

# Clean, then check
clean-check:
    cargo clean
    just check

# Tree of modules and data types
tree:
    cargo modules generate tree --lib --with-types

# Build documentation, open in browser for inspection.
doc:
    cargo doc --all-features --no-deps --open

# Run default application, i.e. kp. Under Windows use triple quotation signs to delimit the operator pipeline.
run ARGS:
    cargo run --features=binary -- {{ARGS}}
# echo 55 12 | just run """geo:in | utm zone=32 | neu:out"""

# Run example based on its unique prefix (e.g. 00, 01, etc.).
run-example EXAMPLE:
    cargo run --example `basename examples/"{{EXAMPLE}}"* .rs`

# Run default application and all examples.
run-all:
    cargo run --features=binary -- --help
    cargo run --example 00-transformations
    cargo run --example 01-geometric_geodesy
    cargo run --example 02-user_defined_macros
    cargo run --example 03-user_defined_operators
    cargo run --example 04-rotating_the_earth
    cargo run --example 05-pq
    cargo run --example 06-user_defined_coordinate_types_and_containers
    cargo run --example 07-examples_from_ruminations

# Compact format log for changelog report
changes:
    git log --pretty=format:"%as: %s (%an)"

# Update CHANGELOG file
changelog:
    git log --pretty=format:"%as: %s (%an)" > CHANGELOG

# Some invisible oddities for general amusement

_sysinfo:
    @echo "This is an {{arch()}} machine, running {{os()}}".

_python:
    #!env python
    print('Hello from python!')
