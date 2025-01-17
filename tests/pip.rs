use crate::common::{apply_lock_filters, cli};
use insta_cmd::assert_cmd_snapshot;
use std::path::Path;
use std::{env, fs};
use tempfile::tempdir;

mod common;

const FIXTURES_PATH: &str = "tests/fixtures/pip";

#[test]
fn test_complete_workflow() {
    let fixture_path = Path::new(FIXTURES_PATH).join("full");
    let requirements_files = [
        "requirements.txt",
        "requirements-dev.txt",
        "requirements-typing.txt",
    ];

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    for file in requirements_files {
        fs::copy(fixture_path.join(file), project_path.join(file)).unwrap();
    }

    apply_lock_filters!();
    assert_cmd_snapshot!(cli()
        .arg(project_path)
        .arg("--dev-requirements-file")
        .arg("requirements-dev.txt")
        .arg("--dev-requirements-file")
        .arg("requirements-typing.txt"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with "uv lock"...
    Using [PYTHON_INTERPRETER]
    warning: No `requires-python` value found in the workspace. Defaulting to `[PYTHON_VERSION]`.
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from pip to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap());

    // Assert that previous package manager files are correctly removed.
    for file in requirements_files {
        assert!(!project_path.join(file).exists());
    }
}

#[test]
fn test_keep_current_data() {
    let fixture_path = Path::new(FIXTURES_PATH).join("full");
    let requirements_files = [
        "requirements.txt",
        "requirements-dev.txt",
        "requirements-typing.txt",
    ];

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    for file in requirements_files {
        fs::copy(fixture_path.join(file), project_path.join(file)).unwrap();
    }

    apply_lock_filters!();
    assert_cmd_snapshot!(cli()
        .arg(project_path)
        .arg("--dev-requirements-file")
        .arg("requirements-dev.txt")
        .arg("--dev-requirements-file")
        .arg("requirements-typing.txt")
        .arg("--keep-current-data"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with "uv lock"...
    Using [PYTHON_INTERPRETER]
    warning: No `requires-python` value found in the workspace. Defaulting to `[PYTHON_VERSION]`.
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from pip to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap());

    // Assert that previous package manager files have not been removed.
    for file in requirements_files {
        assert!(project_path.join(file).exists());
    }
}

#[test]
fn test_skip_lock() {
    let fixture_path = Path::new(FIXTURES_PATH).join("full");
    let requirements_files = [
        "requirements.txt",
        "requirements-dev.txt",
        "requirements-typing.txt",
    ];

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    for file in requirements_files {
        fs::copy(fixture_path.join(file), project_path.join(file)).unwrap();
    }

    assert_cmd_snapshot!(cli()
        .arg(project_path)
        .arg("--dev-requirements-file")
        .arg("requirements-dev.txt")
        .arg("--dev-requirements-file")
        .arg("requirements-typing.txt")
        .arg("--skip-lock"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Successfully migrated project from pip to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap());

    // Assert that previous package manager files are correctly removed.
    for file in requirements_files {
        assert!(!project_path.join(file).exists());
    }

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}

#[test]
fn test_dry_run() {
    let project_path = Path::new(FIXTURES_PATH).join("full");
    let requirements_files = [
        "requirements.txt",
        "requirements-dev.txt",
        "requirements-typing.txt",
    ];

    assert_cmd_snapshot!(cli()
        .arg(&project_path)
        .arg("--dev-requirements-file")
        .arg("requirements-dev.txt")
        .arg("--dev-requirements-file")
        .arg("requirements-typing.txt")
        .arg("--dry-run"));

    // Assert that previous package manager files have not been removed.
    for file in requirements_files {
        assert!(project_path.join(file).exists());
    }

    // Assert that `pyproject.toml` was not created.
    assert!(!project_path.join("pyproject.toml").exists());

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}
