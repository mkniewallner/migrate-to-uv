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
    warning: Could not parse the following dependency specification as a PEP 508 one: file:bar
    warning: Could not parse the following dependency specification as a PEP 508 one: file:./bar
    warning: Could not parse the following dependency specification as a PEP 508 one: git+https://github.com/psf/requests
    warning: Could not parse the following dependency specification as a PEP 508 one: git+https://github.com/psf/requests#egg=requests
    Locking dependencies with "uv lock"...
    Using [PYTHON_INTERPRETER]
    warning: No `requires-python` value found in the workspace. Defaulting to `[PYTHON_VERSION]`.
       Updating https://github.com/encode/uvicorn (HEAD)
        Updated https://github.com/encode/uvicorn ([SHA1])
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from pip to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [project]
    name = ""
    version = "0.0.1"
    dependencies = [
        "arrow==1.3.0",
        "httpx[cli]==0.28.1",
        "uvicorn @ git+https://github.com/encode/uvicorn",
        "requests==2.32.3",
    ]

    [dependency-groups]
    dev = [
        "pytest==8.3.4",
        "ruff==0.8.4",
        "mypy==1.14.1",
        "types-jsonschema==4.23.0.20241208",
    ]

    [tool.uv]
    package = false
    "#);

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
    warning: Could not parse the following dependency specification as a PEP 508 one: file:bar
    warning: Could not parse the following dependency specification as a PEP 508 one: file:./bar
    warning: Could not parse the following dependency specification as a PEP 508 one: git+https://github.com/psf/requests
    warning: Could not parse the following dependency specification as a PEP 508 one: git+https://github.com/psf/requests#egg=requests
    Locking dependencies with "uv lock"...
    Using [PYTHON_INTERPRETER]
    warning: No `requires-python` value found in the workspace. Defaulting to `[PYTHON_VERSION]`.
       Updating https://github.com/encode/uvicorn (HEAD)
        Updated https://github.com/encode/uvicorn ([SHA1])
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from pip to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [project]
    name = ""
    version = "0.0.1"
    dependencies = [
        "arrow==1.3.0",
        "httpx[cli]==0.28.1",
        "uvicorn @ git+https://github.com/encode/uvicorn",
        "requests==2.32.3",
    ]

    [dependency-groups]
    dev = [
        "pytest==8.3.4",
        "ruff==0.8.4",
        "mypy==1.14.1",
        "types-jsonschema==4.23.0.20241208",
    ]

    [tool.uv]
    package = false
    "#);

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
        .arg("--skip-lock"), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    warning: Could not parse the following dependency specification as a PEP 508 one: file:bar
    warning: Could not parse the following dependency specification as a PEP 508 one: file:./bar
    warning: Could not parse the following dependency specification as a PEP 508 one: git+https://github.com/psf/requests
    warning: Could not parse the following dependency specification as a PEP 508 one: git+https://github.com/psf/requests#egg=requests
    Successfully migrated project from pip to uv!
    ");

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [project]
    name = ""
    version = "0.0.1"
    dependencies = [
        "arrow==1.3.0",
        "httpx[cli]==0.28.1",
        "uvicorn @ git+https://github.com/encode/uvicorn",
        "requests==2.32.3",
    ]

    [dependency-groups]
    dev = [
        "pytest==8.3.4",
        "ruff==0.8.4",
        "mypy==1.14.1",
        "types-jsonschema==4.23.0.20241208",
    ]

    [tool.uv]
    package = false
    "#);

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
        .arg("--dry-run"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    warning: Could not parse the following dependency specification as a PEP 508 one: file:bar
    warning: Could not parse the following dependency specification as a PEP 508 one: file:./bar
    warning: Could not parse the following dependency specification as a PEP 508 one: git+https://github.com/psf/requests
    warning: Could not parse the following dependency specification as a PEP 508 one: git+https://github.com/psf/requests#egg=requests
    Migrated pyproject.toml:
    [project]
    name = ""
    version = "0.0.1"
    dependencies = [
        "arrow==1.3.0",
        "httpx[cli]==0.28.1",
        "uvicorn @ git+https://github.com/encode/uvicorn",
        "requests==2.32.3",
    ]

    [dependency-groups]
    dev = [
        "pytest==8.3.4",
        "ruff==0.8.4",
        "mypy==1.14.1",
        "types-jsonschema==4.23.0.20241208",
    ]

    [tool.uv]
    package = false
    "#);

    // Assert that previous package manager files have not been removed.
    for file in requirements_files {
        assert!(project_path.join(file).exists());
    }

    // Assert that `pyproject.toml` was not created.
    assert!(!project_path.join("pyproject.toml").exists());

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}

#[test]
fn test_preserves_existing_project() {
    let project_path = Path::new(FIXTURES_PATH).join("existing_project");

    assert_cmd_snapshot!(cli().arg(&project_path).arg("--dry-run"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Migrated pyproject.toml:
    [project]
    name = "foobar"
    version = "1.0.0"
    requires-python = ">=3.13"
    dependencies = [
        "arrow==1.3.0",
        "httpx[cli]==0.28.1",
        "uvicorn @ git+https://github.com/encode/uvicorn",
    ]

    [tool.uv]
    package = false
    "###);
}

#[test]
fn test_replaces_existing_project() {
    let project_path = Path::new(FIXTURES_PATH).join("existing_project");

    assert_cmd_snapshot!(cli()
        .arg(&project_path)
        .arg("--dry-run")
        .arg("--replace-project-section"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Migrated pyproject.toml:
    [project]
    name = ""
    version = "0.0.1"
    dependencies = [
        "arrow==1.3.0",
        "httpx[cli]==0.28.1",
        "uvicorn @ git+https://github.com/encode/uvicorn",
    ]

    [tool.uv]
    package = false
    "###);
}
