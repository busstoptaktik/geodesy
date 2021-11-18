# Justfile for the Rust Geodesy project.

alias l := list
alias r := run
alias t := test
alias tt := test-all

# Defaults to test.
default: test

# list all justfile targets
list:
    just -l

# Basic test: Just library unit tests. Use target "test-all" or "check" for successively more in depth check ups.
test:
    cargo test --lib

# Unit tests, doc tests, pile test, and compiling of examples
test-all: test-pile
    cargo test

# Check that all tests pass, and that formatting and coding conventions are OK.
check:
    cargo clippy
    cargo fmt -- --check
    cargo test
    cargo doc --no-deps
    cargo package --allow-dirty
    git status

# Clean, then check
clean-check:
    cargo clean
    just check

# Build and install assets
assets:
    zip -r assets.zip assets
    mv assets.zip $LOCALAPPDATA/geodesy
    ls -l $LOCALAPPDATA/geodesy

# Build documentation, open in browser for inspection.
doc:
    cargo doc --no-deps --open

# Run default application.
run ARGS:
    cargo run -- {{ARGS}}


# Run pq application.
pq ARGS:
    cargo run --bin=pq -- {{ARGS}}

# Run example based on its unique prefix (e.g. 00, 01, etc.).
run-example EXAMPLE:
    cargo run --example `basename examples/"{{EXAMPLE}}"* .rs`

# Test the `pile` executable
test-pile:
    touch tests/foobar.pile
    rm tests/foobar.pile
    cargo run --bin pile -- -o tests/foobar.pile tests/foo.raw tests/bar.raw
    diff tests/assets/proj-data/foo.aux tests/assets/expected/foo.aux
    diff tests/assets/proj-data/bar.aux tests/assets/expected/bar.aux
    diff tests/foobar.pile tests/assets/expected/foobar
    rm tests/foobar.pile

# Run default application and all examples.
run-all:
    cargo run -- --help
    cargo run --example 00-transformations
    cargo run --example 01-geometric_geodesy
    cargo run --example 02-user_defined_macros
    cargo run --example 03-user_defined_operators

# Show diff of all 'git add'-ed files
diff:  && stat
    git diff

# Given check passes, commit what has been "git add"-ed
commit: check  &&  stat
    git commit

# Given check passes, add everything and commit all changes
commit-all: check  &&  stat
    git commit -a

# As commit-all but use MESSAGE as commit-message
commit-fast MESSAGE: check  &&  stat
    git commit -a -m "{{MESSAGE}}"

# Git status
stat:
    git status

# Compact format log for changelog report
changes:
    git log --pretty=format:"%as: %s (%an)" > CHANGELOG

# Some invisible oddities for general amusement

_sysinfo:
    @echo "This is an {{arch()}} machine, running {{os()}}".

_python:
    #!env python
    print('Hello from python!')
