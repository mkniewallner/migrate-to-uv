use crate::common::{apply_lock_filters, cli, LockedPackage, UvLock};
use insta_cmd::assert_cmd_snapshot;
use std::path::Path;
use std::{env, fs};
use tempfile::tempdir;

mod common;

const FIXTURES_PATH: &str = "tests/fixtures/poetry";

#[test]
fn test_complete_workflow() {
    let fixture_path = Path::new(FIXTURES_PATH).join("with_lock_file");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    for file in ["poetry.lock", "poetry.toml", "pyproject.toml"] {
        fs::copy(fixture_path.join(file), project_path.join(file)).unwrap();
    }

    apply_lock_filters!();
    assert_cmd_snapshot!(cli().arg(project_path), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with "uv lock"...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Locking dependencies with "uv lock" again to remove constraints...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from Poetry to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap());

    let uv_lock = toml::from_str::<UvLock>(
        fs::read_to_string(project_path.join("uv.lock"))
            .unwrap()
            .as_str(),
    )
    .unwrap();

    // Assert that locked versions in `uv.lock` match what was in `poetry.lock`.
    let uv_lock_packages = uv_lock.package.unwrap();
    let expected_locked_packages = Vec::from([
        LockedPackage {
            name: "arrow".to_string(),
            version: "1.2.3".to_string(),
        },
        LockedPackage {
            name: "factory-boy".to_string(),
            version: "3.2.1".to_string(),
        },
        LockedPackage {
            name: "faker".to_string(),
            version: "33.1.0".to_string(),
        },
        LockedPackage {
            name: "foo".to_string(),
            version: "0.0.1".to_string(),
        },
        LockedPackage {
            name: "mypy".to_string(),
            version: "1.13.0".to_string(),
        },
        LockedPackage {
            name: "mypy-extensions".to_string(),
            version: "1.0.0".to_string(),
        },
        LockedPackage {
            name: "python-dateutil".to_string(),
            version: "2.7.0".to_string(),
        },
        LockedPackage {
            name: "six".to_string(),
            version: "1.15.0".to_string(),
        },
        LockedPackage {
            name: "typing-extensions".to_string(),
            version: "4.6.0".to_string(),
        },
    ]);
    for package in expected_locked_packages {
        assert!(uv_lock_packages.contains(&package));
    }

    // Assert that previous package manager files are correctly removed.
    assert!(!project_path.join("poetry.lock").exists());
    assert!(!project_path.join("poetry.toml").exists());
}

#[test]
fn test_ignore_locked_versions() {
    let fixture_path = Path::new(FIXTURES_PATH).join("with_lock_file");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    for file in ["poetry.lock", "poetry.toml", "pyproject.toml"] {
        fs::copy(fixture_path.join(file), project_path.join(file)).unwrap();
    }

    apply_lock_filters!();
    assert_cmd_snapshot!(cli().arg(project_path).arg("--ignore-locked-versions"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with "uv lock"...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from Poetry to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap());

    let uv_lock = toml::from_str::<UvLock>(
        fs::read_to_string(project_path.join("uv.lock"))
            .unwrap()
            .as_str(),
    )
    .unwrap();

    let mut arrow: Option<LockedPackage> = None;
    let mut typing_extensions: Option<LockedPackage> = None;
    for package in uv_lock.package.unwrap() {
        if package.name == "arrow" {
            arrow = Some(package);
        } else if package.name == "typing-extensions" {
            typing_extensions = Some(package);
        }
    }

    // Assert that locked versions are different that what was in `poetry.lock`.
    assert_ne!(arrow.unwrap().version, "1.2.3");
    assert_ne!(typing_extensions.unwrap().version, "4.6.0");

    // Assert that previous package manager files are correctly removed.
    assert!(!project_path.join("poetry.lock").exists());
    assert!(!project_path.join("poetry.toml").exists());
}

#[test]
fn test_keep_current_data() {
    let fixture_path = Path::new(FIXTURES_PATH).join("with_lock_file");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    for file in ["poetry.lock", "poetry.toml", "pyproject.toml"] {
        fs::copy(fixture_path.join(file), project_path.join(file)).unwrap();
    }

    apply_lock_filters!();
    assert_cmd_snapshot!(cli().arg(project_path).arg("--keep-current-data"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with "uv lock"...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Locking dependencies with "uv lock" again to remove constraints...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from Poetry to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap());

    // Assert that previous package manager files have not been removed.
    assert!(project_path.join("poetry.lock").exists());
    assert!(project_path.join("poetry.toml").exists());
}

#[test]
fn test_dependency_groups_strategy_include_in_dev() {
    let fixture_path = Path::new(FIXTURES_PATH).join("with_lock_file");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    for file in ["poetry.lock", "poetry.toml", "pyproject.toml"] {
        fs::copy(fixture_path.join(file), project_path.join(file)).unwrap();
    }

    apply_lock_filters!();
    assert_cmd_snapshot!(cli()
        .arg(project_path)
        .arg("--dependency-groups-strategy")
        .arg("include-in-dev"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with "uv lock"...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Locking dependencies with "uv lock" again to remove constraints...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from Poetry to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap());

    // Assert that previous package manager files are correctly removed.
    assert!(!project_path.join("poetry.lock").exists());
    assert!(!project_path.join("poetry.toml").exists());
}

#[test]
fn test_dependency_groups_strategy_keep_existing() {
    let fixture_path = Path::new(FIXTURES_PATH).join("with_lock_file");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    for file in ["poetry.lock", "poetry.toml", "pyproject.toml"] {
        fs::copy(fixture_path.join(file), project_path.join(file)).unwrap();
    }

    apply_lock_filters!();
    assert_cmd_snapshot!(cli()
        .arg(project_path)
        .arg("--dependency-groups-strategy")
        .arg("keep-existing"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with "uv lock"...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Locking dependencies with "uv lock" again to remove constraints...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from Poetry to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap());

    // Assert that previous package manager files are correctly removed.
    assert!(!project_path.join("poetry.lock").exists());
    assert!(!project_path.join("poetry.toml").exists());
}

#[test]
fn test_dependency_groups_strategy_merge_into_dev() {
    let fixture_path = Path::new(FIXTURES_PATH).join("with_lock_file");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    for file in ["poetry.lock", "poetry.toml", "pyproject.toml"] {
        fs::copy(fixture_path.join(file), project_path.join(file)).unwrap();
    }

    apply_lock_filters!();
    assert_cmd_snapshot!(cli()
        .arg(project_path)
        .arg("--dependency-groups-strategy")
        .arg("merge-into-dev"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with "uv lock"...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Locking dependencies with "uv lock" again to remove constraints...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from Poetry to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap());

    // Assert that previous package manager files are correctly removed.
    assert!(!project_path.join("poetry.lock").exists());
    assert!(!project_path.join("poetry.toml").exists());
}

#[test]
fn test_skip_lock() {
    let fixture_path = Path::new(FIXTURES_PATH).join("with_lock_file");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    for file in ["poetry.lock", "poetry.toml", "pyproject.toml"] {
        fs::copy(fixture_path.join(file), project_path.join(file)).unwrap();
    }

    assert_cmd_snapshot!(cli().arg(project_path).arg("--skip-lock"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Successfully migrated project from Poetry to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap());

    // Assert that previous package manager files are correctly removed.
    assert!(!project_path.join("poetry.lock").exists());
    assert!(!project_path.join("poetry.toml").exists());

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}

#[test]
fn test_skip_lock_full() {
    let fixture_path = Path::new(FIXTURES_PATH).join("full");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    fs::copy(
        fixture_path.join("pyproject.toml"),
        project_path.join("pyproject.toml"),
    )
    .unwrap();

    assert_cmd_snapshot!(cli().arg(project_path).arg("--skip-lock"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    warning: Could not find dependency "non-existing-dependency" listed in "extra-with-non-existing-dependencies" extra.
    Successfully migrated project from Poetry to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap());

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}

#[test]
fn test_dry_run() {
    let project_path = Path::new(FIXTURES_PATH).join("with_lock_file");
    let pyproject = fs::read_to_string(project_path.join("pyproject.toml")).unwrap();

    assert_cmd_snapshot!(cli().arg(&project_path).arg("--dry-run"));

    // Assert that `pyproject.toml` was not updated.
    assert_eq!(
        pyproject,
        fs::read_to_string(project_path.join("pyproject.toml")).unwrap()
    );

    // Assert that previous package manager files have not been removed.
    assert!(project_path.join("poetry.lock").exists());
    assert!(project_path.join("poetry.toml").exists());

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}

#[test]
fn test_dry_run_minimal() {
    let project_path = Path::new(FIXTURES_PATH).join("minimal");
    let pyproject = fs::read_to_string(project_path.join("pyproject.toml")).unwrap();

    assert_cmd_snapshot!(cli().arg(&project_path).arg("--dry-run"));

    // Assert that `pyproject.toml` was not updated.
    assert_eq!(
        pyproject,
        fs::read_to_string(project_path.join("pyproject.toml")).unwrap()
    );

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}
