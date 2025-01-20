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

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r###"
    [project]
    name = ""
    version = "0.0.1"
    dependencies = [
        "anyio==4.7.0",
        "arrow==1.3.0",
        "certifi==2024.12.14",
        "click==8.1.8",
        "h11==0.14.0",
        "httpcore==1.0.7",
        "httpx==0.28.1",
        "idna==3.10",
        "markdown-it-py==3.0.0",
        "mdurl==0.1.2",
        "pygments==2.18.0",
        "python-dateutil==2.9.0.post0",
        "rich==13.9.4",
        "six==1.17.0",
        "sniffio==1.3.1",
        "types-python-dateutil==2.9.0.20241206",
        "uvicorn @ git+https://github.com/encode/uvicorn",
        "zstandard==0.23.0",
    ]

    [dependency-groups]
    dev = [
        "iniconfig==2.0.0",
        "packaging==24.2",
        "pluggy==1.5.0",
        "pytest==8.3.4",
        "ruff==0.8.4",
        "attrs==24.3.0",
        "mypy==1.14.1",
        "mypy-extensions==1.0.0",
        "referencing==0.35.1",
        "rpds-py==0.22.3",
        "types-jsonschema==4.23.0.20241208",
        "typing-extensions==4.12.2",
    ]

    [tool.uv]
    package = false
    "###);

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

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r###"
    [project]
    name = ""
    version = "0.0.1"
    dependencies = [
        "anyio==4.7.0",
        "arrow==1.3.0",
        "certifi==2024.12.14",
        "click==8.1.8",
        "h11==0.14.0",
        "httpcore==1.0.7",
        "httpx==0.28.1",
        "idna==3.10",
        "markdown-it-py==3.0.0",
        "mdurl==0.1.2",
        "pygments==2.18.0",
        "python-dateutil==2.9.0.post0",
        "rich==13.9.4",
        "six==1.17.0",
        "sniffio==1.3.1",
        "types-python-dateutil==2.9.0.20241206",
        "uvicorn @ git+https://github.com/encode/uvicorn",
        "zstandard==0.23.0",
    ]

    [dependency-groups]
    dev = [
        "iniconfig==2.0.0",
        "packaging==24.2",
        "pluggy==1.5.0",
        "pytest==8.3.4",
        "ruff==0.8.4",
        "attrs==24.3.0",
        "mypy==1.14.1",
        "mypy-extensions==1.0.0",
        "referencing==0.35.1",
        "rpds-py==0.22.3",
        "types-jsonschema==4.23.0.20241208",
        "typing-extensions==4.12.2",
    ]

    [tool.uv]
    package = false
    "###);

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

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r###"
    [project]
    name = ""
    version = "0.0.1"
    dependencies = [
        "anyio==4.7.0",
        "arrow==1.3.0",
        "certifi==2024.12.14",
        "click==8.1.8",
        "h11==0.14.0",
        "httpcore==1.0.7",
        "httpx==0.28.1",
        "idna==3.10",
        "markdown-it-py==3.0.0",
        "mdurl==0.1.2",
        "pygments==2.18.0",
        "python-dateutil==2.9.0.post0",
        "rich==13.9.4",
        "six==1.17.0",
        "sniffio==1.3.1",
        "types-python-dateutil==2.9.0.20241206",
        "uvicorn @ git+https://github.com/encode/uvicorn",
        "zstandard==0.23.0",
    ]

    [dependency-groups]
    dev = [
        "iniconfig==2.0.0",
        "packaging==24.2",
        "pluggy==1.5.0",
        "pytest==8.3.4",
        "ruff==0.8.4",
        "attrs==24.3.0",
        "mypy==1.14.1",
        "mypy-extensions==1.0.0",
        "referencing==0.35.1",
        "rpds-py==0.22.3",
        "types-jsonschema==4.23.0.20241208",
        "typing-extensions==4.12.2",
    ]

    [tool.uv]
    package = false
    "###);

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
        .arg("--dry-run"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Migrated pyproject.toml:
    [project]
    name = ""
    version = "0.0.1"
    dependencies = [
        "anyio==4.7.0",
        "arrow==1.3.0",
        "certifi==2024.12.14",
        "click==8.1.8",
        "h11==0.14.0",
        "httpcore==1.0.7",
        "httpx==0.28.1",
        "idna==3.10",
        "markdown-it-py==3.0.0",
        "mdurl==0.1.2",
        "pygments==2.18.0",
        "python-dateutil==2.9.0.post0",
        "rich==13.9.4",
        "six==1.17.0",
        "sniffio==1.3.1",
        "types-python-dateutil==2.9.0.20241206",
        "uvicorn @ git+https://github.com/encode/uvicorn",
        "zstandard==0.23.0",
    ]

    [dependency-groups]
    dev = [
        "iniconfig==2.0.0",
        "packaging==24.2",
        "pluggy==1.5.0",
        "pytest==8.3.4",
        "ruff==0.8.4",
        "attrs==24.3.0",
        "mypy==1.14.1",
        "mypy-extensions==1.0.0",
        "referencing==0.35.1",
        "rpds-py==0.22.3",
        "types-jsonschema==4.23.0.20241208",
        "typing-extensions==4.12.2",
    ]

    [tool.uv]
    package = false
    "###);

    // Assert that previous package manager files have not been removed.
    for file in requirements_files {
        assert!(project_path.join(file).exists());
    }

    // Assert that `pyproject.toml` was not created.
    assert!(!project_path.join("pyproject.toml").exists());

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}
