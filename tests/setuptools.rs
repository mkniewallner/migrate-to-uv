use crate::common::cli;
use insta_cmd::assert_cmd_snapshot;
use std::path::Path;
use std::{env, fs};
use tempfile::tempdir;

mod common;

const FIXTURES_PATH: &str = "tests/fixtures/setuptools";

#[test]
fn test_skip_lock_full() {
    let fixture_path = Path::new(FIXTURES_PATH).join("full");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    fs::copy(
        fixture_path.join("setup.cfg"),
        project_path.join("setup.cfg"),
    )
    .unwrap();

    assert_cmd_snapshot!(cli().arg(project_path).arg("--skip-lock"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Successfully migrated project from Setuptools to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r###""###);

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}

#[test]
fn test_dry_run() {
    let project_path = Path::new(FIXTURES_PATH).join("full");

    assert_cmd_snapshot!(cli().arg(&project_path).arg("--dry-run"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Migrated pyproject.toml:
    [project]
    name = "foobar"
    description = "A fabulous project."
    authors = [{ name = "John Doe", email = "john.doe@example.com" }]
    requires-python = ">=3.11"
    license = "MIT"
    maintainers = [{ name = "Dohn Joe", email = "dohn.joe@example.com" }]
    keywords = [
        "foo",
        "bar",
        "foobar",
    ]
    classifiers = [
        "Development Status :: 3 - Alpha",
        "Environment :: Console",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Operating System :: OS Independent",
    ]
    dependencies = ["arrow>=1.2.3"]

    [project.optional-dependencies]
    pdf = [
        "foo>=1.0.0",
        "bar",
    ]
    rest = ["baz>=2.0.2"]
    "###);

    // Assert that previous package manager files have not been removed.
    assert!(project_path.join("setup.cfg").exists());

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}
