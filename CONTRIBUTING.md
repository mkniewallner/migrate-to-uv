# Contributing

## Setup

[Rust](https://rustup.rs/) is required to build the project.

## Linting and formatting

The project uses several tools from the Rust ecosystem to check for common linting issues, and ensure that the code is
correctly formatted, like:

- [clippy](https://doc.rust-lang.org/clippy/) for linting
- [rustfmt](https://rust-lang.github.io/rustfmt/) for formatting

[pre-commit](https://pre-commit.com/) is used to ensure that all tools are run at commit time. You can install hooks in
the project with:

```bash
pre-commit install
```

This will automatically run the relevant git hooks based on the files that are modified whenever you commit.

You can also run all hooks manually without committing with:

```bash
pre-commit run --all-files
```

## Testing

Both unit and integration tests are used to ensure that the code work as intended. They can be run with:

```bash
make test
```

Unit tests are located in modules, alongside the code, under `src` directory, and can be run with:

```bash
make test-unit
```

Integration tests are located under `tests` directory, and can be run with:

```bash
make test-integration
```

As integration tests depend on [uv](https://docs.astral.sh/uv/) for performing locking, make sure that it is present on
your machine before running them.

### Snapshots

Both unit and integration tests use snapshot testing through [insta](https://insta.rs/), to assert things like the
content of files or command line outputs. Those snapshots can either be asserted right into the code, or against files
stored in `snapshots` directories, for instance:

```rust
#[test]
fn test_with_snapshots() {
    // Inline snapshot
    insta::assert_snapshot!(foo(), @r###"
        [project]
        name = "foo"
        version = "0.0.1"
        "###);
    
    // External snapshot, stored under `snapshots` directory
    insta::assert_snapshot!(foo());
}
```

In both cases, if you update code that changes the output of snapshots, you will be prompted to review the updated
snapshots with:

```bash
cargo insta review
```

You can then accept the changes, if they look correct according to the changed code.

## Documentation

Documentation is built using [mkdocs](https://www.mkdocs.org/)
and [mkdocs-material](https://squidfunk.github.io/mkdocs-material/).

It can be run locally with [uv](https://docs.astral.sh/uv/) by using:

```bash
make doc-serve
```
