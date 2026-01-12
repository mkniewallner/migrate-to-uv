use crate::common::{LockedPackage, UvLock, apply_filters, cli};
use dircpy::copy_dir;
use flate2::read::GzDecoder;
use insta_cmd::assert_cmd_snapshot;
use std::fs;
use std::fs::{File, remove_dir_all};
use std::path::Path;
use std::process::{Command, Stdio};
use tar::Archive;
use tempfile::tempdir;
use walkdir::WalkDir;
use zip::ZipArchive;

mod common;

const FIXTURES_PATH: &str = "tests/fixtures/poetry";

fn get_tar_gz_entries(path: &Path, file_name: &str) -> Vec<String> {
    let extract_path = path.join("tar_gz_content");
    let mut archive = Archive::new(GzDecoder::new(File::open(path.join(file_name)).unwrap()));
    archive.unpack(&extract_path).unwrap();

    let mut entries: Vec<String> = WalkDir::new(&extract_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.metadata().unwrap().is_file())
        .map(|e| {
            e.path()
                .strip_prefix(&extract_path)
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    entries.sort_unstable();

    entries
}

fn get_zip_entries(path: &Path, file_name: &str) -> Vec<String> {
    let extract_path = path.join("zip_content");
    let mut archive = ZipArchive::new(File::open(path.join(file_name)).unwrap()).unwrap();
    archive.extract(&extract_path).unwrap();

    let mut entries: Vec<String> = WalkDir::new(path.join("zip_content"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.metadata().unwrap().is_file())
        .map(|e| {
            e.path()
                .strip_prefix(&extract_path)
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    entries.sort_unstable();

    entries
}

#[test]
fn test_complete_workflow() {
    let fixture_path = Path::new(FIXTURES_PATH).join("with_lock_file");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(fixture_path, project_path).unwrap();

    apply_filters!();
    assert_cmd_snapshot!(cli().arg(project_path), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with constraints from existing lock file(s) using "uv lock"...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Locking dependencies again using "uv lock" to remove constraints...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from Poetry to uv!
    "#);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = ">=3.11,<4"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]
    profiling = ["pyinstrument>=5.0.2,<6"]

    [tool.uv]
    package = false
    default-groups = [
        "dev",
        "typing",
    ]
    "#);

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
fn test_complete_workflow_pep_621_no_poetry_section() {
    let fixture_path = Path::new(FIXTURES_PATH).join("pep_621_no_poetry_section_with_lock_file");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(fixture_path, project_path).unwrap();

    apply_filters!();
    assert_cmd_snapshot!(cli().arg(project_path), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with constraints from existing lock file(s) using "uv lock"...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Locking dependencies again using "uv lock" to remove constraints...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from Poetry to uv!
    "#);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [build-system]
    requires = ["uv_build>=[LOWER_BOUND],<[UPPER_BOUND]"]
    build-backend = "uv_build"

    [project]
    name = "foo"
    version = "0.1.0"
    description = "A fabulous project."
    requires-python = ">=3.11"
    classifiers = [
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Python :: 3.13",
        "Programming Language :: Python :: 3.14",
    ]
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]
    profiling = ["pyinstrument>=5.0.2,<6"]
    "#);

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

    copy_dir(fixture_path, project_path).unwrap();

    apply_filters!();
    assert_cmd_snapshot!(cli().arg(project_path).arg("--ignore-locked-versions"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies using "uv lock"...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from Poetry to uv!
    "#);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = ">=3.11,<4"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]
    profiling = ["pyinstrument>=5.0.2,<6"]

    [tool.uv]
    package = false
    default-groups = [
        "dev",
        "typing",
    ]
    "#);

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

    copy_dir(fixture_path, project_path).unwrap();

    apply_filters!();
    assert_cmd_snapshot!(cli().arg(project_path).arg("--keep-current-data"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with constraints from existing lock file(s) using "uv lock"...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Locking dependencies again using "uv lock" to remove constraints...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from Poetry to uv!
    "#);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = ">=3.11,<4"
    dependencies = ["arrow>=1.2.3,<2"]

    [tool.poetry]
    package-mode = false
    name = "foo"

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]
    profiling = ["pyinstrument>=5.0.2,<6"]

    [tool.poetry.dependencies]
    python = "^3.11"
    arrow = "^1.2.3"

    [tool.uv]
    package = false
    default-groups = [
        "dev",
        "typing",
    ]

    [tool.poetry.group.dev.dependencies]
    factory-boy = "^3.2.1"

    [tool.poetry.group.typing.dependencies]
    mypy = "^1.13.0"

    [tool.poetry.group.profiling]
    optional = true

    [tool.poetry.group.profiling.dependencies]
    pyinstrument = "^5.0.2"
    "#);

    // Assert that previous package manager files have not been removed.
    assert!(project_path.join("poetry.lock").exists());
    assert!(project_path.join("poetry.toml").exists());
}

#[test]
fn test_dependency_groups_strategy_include_in_dev() {
    let fixture_path = Path::new(FIXTURES_PATH).join("with_lock_file");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(fixture_path, project_path).unwrap();

    apply_filters!();
    assert_cmd_snapshot!(cli()
        .arg(project_path)
        .arg("--dependency-groups-strategy")
        .arg("include-in-dev"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with constraints from existing lock file(s) using "uv lock"...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Locking dependencies again using "uv lock" to remove constraints...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from Poetry to uv!
    "#);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = ">=3.11,<4"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = [
        "factory-boy>=3.2.1,<4",
        { include-group = "typing" },
    ]
    typing = ["mypy>=1.13.0,<2"]
    profiling = ["pyinstrument>=5.0.2,<6"]

    [tool.uv]
    package = false
    "#);

    // Assert that previous package manager files are correctly removed.
    assert!(!project_path.join("poetry.lock").exists());
    assert!(!project_path.join("poetry.toml").exists());
}

#[test]
fn test_dependency_groups_strategy_keep_existing() {
    let fixture_path = Path::new(FIXTURES_PATH).join("with_lock_file");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(fixture_path, project_path).unwrap();

    apply_filters!();
    assert_cmd_snapshot!(cli()
        .arg(project_path)
        .arg("--dependency-groups-strategy")
        .arg("keep-existing"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with constraints from existing lock file(s) using "uv lock"...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Locking dependencies again using "uv lock" to remove constraints...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from Poetry to uv!
    "#);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = ">=3.11,<4"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]
    profiling = ["pyinstrument>=5.0.2,<6"]

    [tool.uv]
    package = false
    "#);

    // Assert that previous package manager files are correctly removed.
    assert!(!project_path.join("poetry.lock").exists());
    assert!(!project_path.join("poetry.toml").exists());
}

#[test]
fn test_dependency_groups_strategy_merge_into_dev() {
    let fixture_path = Path::new(FIXTURES_PATH).join("with_lock_file");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(fixture_path, project_path).unwrap();

    apply_filters!();
    assert_cmd_snapshot!(cli()
        .arg(project_path)
        .arg("--dependency-groups-strategy")
        .arg("merge-into-dev"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with constraints from existing lock file(s) using "uv lock"...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Locking dependencies again using "uv lock" to remove constraints...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from Poetry to uv!
    "#);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = ">=3.11,<4"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = [
        "factory-boy>=3.2.1,<4",
        "mypy>=1.13.0,<2",
    ]
    profiling = ["pyinstrument>=5.0.2,<6"]

    [tool.uv]
    package = false
    "#);

    // Assert that previous package manager files are correctly removed.
    assert!(!project_path.join("poetry.lock").exists());
    assert!(!project_path.join("poetry.toml").exists());
}

#[test]
fn test_skip_lock() {
    let fixture_path = Path::new(FIXTURES_PATH).join("with_lock_file");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(fixture_path, project_path).unwrap();

    assert_cmd_snapshot!(cli().arg(project_path).arg("--skip-lock"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Successfully migrated project from Poetry to uv!
    "###);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = ">=3.11,<4"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]
    profiling = ["pyinstrument>=5.0.2,<6"]

    [tool.uv]
    package = false
    default-groups = [
        "dev",
        "typing",
    ]
    "#);

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

    copy_dir(fixture_path, project_path).unwrap();

    assert_cmd_snapshot!(cli().arg(project_path).arg("--skip-lock"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Successfully migrated project from Poetry to uv!

    warning: The following warnings occurred during the migration:
    warning: - Migrating build backend to Hatch because package distribution metadata is too complex for uv.
    warning: - Could not find dependency "non-existing-dependency" listed in "extra-with-non-existing-dependencies" extra.
    warning: - Build backend was migrated to Hatch. It is highly recommended to manually check that files included in the source distribution and wheels are the same than before the migration.
    "#);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [build-system]
    requires = ["hatchling"]
    build-backend = "hatchling.build"

    [project]
    name = "foobar"
    version = "0.1.0"
    description = "A fabulous project."
    authors = [{ name = "John Doe", email = "john.doe@example.com" }]
    requires-python = ">=3.11,<4"
    readme = "README.md"
    license = "MIT"
    maintainers = [
        { name = "Dohn Joe", email = "dohn.joe@example.com" },
        { name = "Johd Noe" },
    ]
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
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Python :: 3.13",
        "Programming Language :: Python :: 3.14",
    ]
    dependencies = [
        "caret>=1.2.3,<2",
        "caret-2>=1.2,<2",
        "caret-3>=1,<2",
        "caret-4>=0.2.3,<0.3",
        "caret-5>=0.0.3,<0.0.4",
        "caret-6>=0.0,<0.1",
        "caret-7>=0,<1",
        "caret-8>=1.2.3.4,<2",
        "caret-9>=0.1.2.3,<0.2",
        "caret-pre-release>=1.2.3b1,<2",
        "caret-and-caret>=1.0,<2,>=1.0,<2",
        "caret-and-pep440>=1.0,<2,<1.3",
        "caret-and-pep440-2>=1.0,<2,>=1.1,<1.3",
        "caret-and-pep440-3>=1.0,<2,>=1.1,>=1.2,!=1.2.2,<1.3",
        "caret-and-pep440-whitepsaces>=1.0,<2,>=   1.1,>=  1.2,!=   1.2.2,< 1.3",
        "caret-and-caret-and-pep440>=1.0,<2,>=1.1,<2,<1.2",
        "tilde~=1.2.3",
        "tilde-2>=1.2,<1.3",
        "tilde-3>=1,<2",
        "tilde-4~=1.2.3.4",
        "tilde-pre-release~=1.2.3b1",
        "tilde-and-tilde>=1.0,<1.1,>=1.0,<1.1",
        "tilde-and-pep440>=1.0,<1.1,<1.3",
        "tilde-and-pep440-2>=1.0,<1.1,>=1.0,<1.1",
        "tilde-and-pep440-3>=1.0,<1.1,>=1.0,>=1.0.1,!=1.0.2,<1.3",
        "tilde-and-pep440-whitepsaces>=1.0,<1.1,>=   1.0,>=  1.0.1,!=   1.0.2,< 1.3",
        "tilde-and-tilde-and-pep440>=1.0,<1.1,~=1.0.1,<1.2",
        "exact==1.2.3",
        "exact-2==1.2.3",
        "star",
        "star-2==1.*",
        "star-3==1.2.*",
        "pep440>=1.2.3",
        "with-version-only==1.2.3",
        "with-extras[asyncio, postgresql_asyncpg]==1.2.3",
        "with-markers==1.2.3 ; python_version <= '3.11' or sys_platform == 'win32'",
        "with-platform==1.2.3 ; sys_platform == 'darwin'",
        "with-pipe-delimited-platform==1.2.3 ; sys_platform == 'darwin' or sys_platform == 'linux'",
        "with-double-pipe-delimited-platform==1.2.3 ; sys_platform == 'darwin' or sys_platform == 'linux' or sys_platform == 'windows'",
        "with-double-pipe-delimited-platform-and-spaces==1.2.3 ; sys_platform == 'darwin' or sys_platform == 'linux' or sys_platform == 'windows'",
        "with-markers-python-platform==1.2.3 ; python_full_version >= '3.11' and python_full_version < '3.12' and platform_python_implementation == 'CPython' or platform_python_implementation == 'Jython' and sys_platform == 'darwin'",
        "with-source==1.2.3",
        "python-restricted==1.2.3 ; python_full_version >= '3.11' and python_full_version < '4'",
        "python-restricted-2==1.2.3 ; python_full_version >= '3.11' and python_full_version < '3.12'",
        "python-restricted-3==1.2.3 ; python_full_version > '3.11'",
        "python-restricted-4==1.2.3 ; python_full_version >= '3.11'",
        "python-restricted-5==1.2.3 ; python_full_version < '3.11'",
        "python-restricted-6==1.2.3 ; python_full_version <= '3.11'",
        "python-restricted-7==1.2.3 ; python_full_version > '3.11' and python_full_version < '3.13'",
        "python-restricted-full-version==1.2.3 ; python_full_version >= '3.11.2' and python_full_version < '4'",
        "python-restricted-exact-patch-version==1.2.3 ; python_full_version == '3.11.2'",
        "python-restricted-exact-minor-version==1.2.3 ; python_full_version == '3.11.*'",
        "python-restricted-exact-major-version==1.2.3 ; python_full_version == '3.0.*'",
        "python-restricted-equal-patch-version==1.2.3 ; python_full_version == '3.11.2'",
        "python-restricted-equal-minor-version==1.2.2 ; python_full_version == '3.11.*'",
        "python-restricted-equal-major-version==1.2.3 ; python_full_version == '3.0.*'",
        "python-restricted-with-source==1.2.3 ; python_full_version > '3.11' and python_full_version < '3.13'",
        "whitespaces>=3.2,<4",
        "whitespaces-2     >   3.11,     <=     3.13    ",
        "optional-not-in-extra==1.2.3",
        "local-package",
        "local-package-2",
        "local-package-editable",
        "url-dep",
        "git",
        "git-branch",
        "git-rev",
        "git-tag",
        "git-subdirectory",
        "multiple-constraints-python-version>=2 ; python_full_version >= '3.11'",
        "multiple-constraints-python-version<2 ; python_full_version < '3.11'",
        "multiple-constraints-platform-version>=2 ; sys_platform == 'darwin'",
        "multiple-constraints-platform-version<2 ; sys_platform == 'linux'",
        "multiple-constraints-markers-version>=2 ; platform_python_implementation == 'CPython'",
        "multiple-constraints-markers-version<2 ; platform_python_implementation != 'CPython'",
        "multiple-constraints-python-platform-markers-version>=2 ; python_full_version >= '3.11' and platform_python_implementation == 'CPython' and sys_platform == 'darwin'",
        "multiple-constraints-python-platform-markers-version<2 ; python_full_version < '3.11' and platform_python_implementation != 'CPython' and sys_platform == 'linux'",
        "multiple-constraints-python-source",
        "multiple-constraints-platform-source",
        "multiple-constraints-markers-source",
        "multiple-constraints-python-platform-markers-source",
    ]

    [project.optional-dependencies]
    extra = ["dep-in-extra==1.2.3"]
    extra-2 = [
        "dep-in-extra==1.2.3",
        "optional-in-extra==1.2.3",
    ]
    extra-with-non-existing-dependencies = []

    [project.urls]
    Homepage = "https://homepage.example.com"
    Repository = "https://repository.example.com"
    Documentation = "https://docs.example.com"
    "First link" = "https://first.example.com"
    "Another link" = "https://another.example.com"

    [project.scripts]
    console-script = "foo:run"
    console-script-2 = "override_bar:run"
    console-script-3 = "foobar:run"

    [project.gui-scripts]
    gui-script = "gui:run"

    [project.entry-points.some-scripts]
    a-script = "a_script:run"
    another-script = "another_script:run"

    [project.entry-points.other-scripts]
    a-script = "another_script:run"
    yet-another-script = "yet_another_scripts:run"

    [dependency-groups]
    dev = [
        "dev-legacy==1.2.3",
        "dev-legacy-2==1.2.3",
        "dev-dep==1.2.3",
    ]
    typing = ["typing-dep==1.2.3"]
    profiling = ["pyinstrument==5.0.2"]

    [tool.uv]
    package = false
    default-groups = [
        "dev",
        "typing",
    ]

    [[tool.uv.index]]
    name = "PyPI"
    url = "https://pypi.org/simple/"
    default = true

    [[tool.uv.index]]
    name = "secondary"
    url = "https://secondary.example.com/simple/"

    [[tool.uv.index]]
    name = "supplemental"
    url = "https://supplemental.example.com/simple/"

    [[tool.uv.index]]
    name = "explicit"
    url = "https://explicit.example.com/simple/"
    explicit = true

    [[tool.uv.index]]
    name = "default"
    url = "https://default.example.com/simple/"
    default = true

    [tool.uv.sources]
    with-source = { index = "supplemental" }
    python-restricted-with-source = { index = "supplemental" }
    local-package = { path = "package/" }
    local-package-2 = { path = "package/dist/package-0.1.0.tar.gz", editable = false }
    local-package-editable = { path = "editable-package/", editable = true }
    url-dep = { url = "https://example.com/package-0.0.1.tar.gz" }
    git = { git = "https://example.com/foo/bar" }
    git-branch = { git = "https://example.com/foo/bar", branch = "foo" }
    git-rev = { git = "https://example.com/foo/bar", rev = "1234567" }
    git-tag = { git = "https://example.com/foo/bar", tag = "v1.2.3" }
    git-subdirectory = { git = "https://example.com/foo/bar", subdirectory = "directory" }
    multiple-constraints-python-source = [
        { url = "https://example.com/foo-1.2.3-py3-none-any.whl", marker = "python_full_version >= '3.11'" },
        { git = "https://example.com/foo/bar", tag = "v1.2.3", marker = "python_full_version < '3.11'" },
    ]
    multiple-constraints-platform-source = [
        { url = "https://example.com/foo-1.2.3-py3-none-any.whl", marker = "sys_platform == 'darwin'" },
        { git = "https://example.com/foo/bar", tag = "v1.2.3", marker = "sys_platform == 'linux'" },
    ]
    multiple-constraints-markers-source = [
        { url = "https://example.com/foo-1.2.3-py3-none-any.whl", marker = "platform_python_implementation == 'CPython'" },
        { git = "https://example.com/foo/bar", tag = "v1.2.3", marker = "platform_python_implementation != 'CPython'" },
    ]
    multiple-constraints-python-platform-markers-source = [
        { url = "https://example.com/foo-1.2.3-py3-none-any.whl", marker = "python_full_version >= '3.11' and platform_python_implementation == 'CPython' and sys_platform == 'darwin'" },
        { index = "supplemental", marker = "python_full_version < '3.11' and platform_python_implementation != 'CPython' and sys_platform == 'linux'" },
    ]

    [tool.hatch.build.targets.sdist]
    include = [
        "packages_sdist_wheel",
        "packages_sdist_wheel_2",
        "packages_sdist",
        "packages_sdist_2",
        "from/packages_from",
        "packages_to",
        "from/packages_from_to",
        "text_file_sdist_wheel.txt",
        "text_file_sdist.txt",
    ]
    exclude = [
        "exclude_sdist_wheel",
        "exclude_sdist_wheel_2",
    ]

    [tool.hatch.build.targets.sdist.force-include]
    include_sdist = "include_sdist"
    include_sdist_2 = "include_sdist_2"
    include_sdist_3 = "include_sdist_3"
    include_sdist_4 = "include_sdist_4"
    include_sdist_wheel = "include_sdist_wheel"

    [tool.hatch.build.targets.wheel]
    include = [
        "packages_sdist_wheel",
        "packages_sdist_wheel_2",
        "packages_wheel",
        "packages_wheel_2",
        "from/packages_from",
        "packages_to",
        "from/packages_from_to",
        "text_file_sdist_wheel.txt",
        "text_file_wheel.txt",
    ]
    exclude = [
        "exclude_sdist_wheel",
        "exclude_sdist_wheel_2",
    ]

    [tool.hatch.build.targets.wheel.force-include]
    include_sdist_wheel = "include_sdist_wheel"
    include_wheel = "include_wheel"
    include_wheel_2 = "include_wheel_2"

    [tool.hatch.build.targets.wheel.sources]
    "from/packages_from" = "packages_from"
    packages_to = "to/packages_to"
    "from/packages_from_to" = "to/packages_from_to"

    # This comment should be preserved.
    [tool.ruff]
    fix = true

    # This comment should be preserved.
    [tool.ruff.lint]
    # This comment should be preserved.
    fixable = ["I", "UP"]

    # This comment should be preserved.
    [tool.ruff.format]
    preview = true

    # This comment should be preserved.
    [tool.mypy]
    files = [
        "foo",
        "tests", # handwritten tests
    ]
    # This comment should be preserved.

    # This comment should be preserved.
    warn_unused_configs = true

    ## This comment should be preserved.
    disallow_subclassing_any = true

    ### This comment should be preserved.
    #disallow_untyped_calls = true
    #
    ### foo
    #no_implicit_reexport = true
    #

    [[tool.mypy.overrides]]
    module = ["foo"]
    warn_unused_ignores = true
    "#);

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}

#[test]
fn test_dry_run() {
    let project_path = Path::new(FIXTURES_PATH).join("with_lock_file");
    let pyproject = fs::read_to_string(project_path.join("pyproject.toml")).unwrap();

    assert_cmd_snapshot!(cli().arg(&project_path).arg("--dry-run"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Migrated pyproject.toml:
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = ">=3.11,<4"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]
    profiling = ["pyinstrument>=5.0.2,<6"]

    [tool.uv]
    package = false
    default-groups = [
        "dev",
        "typing",
    ]
    "#);

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

    assert_cmd_snapshot!(cli().arg(&project_path).arg("--dry-run"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Migrated pyproject.toml:
    [project]
    name = "foobar"
    version = "0.0.1"

    [tool.ruff]
    fix = true

    [tool.ruff.format]
    preview = true
    "###);

    // Assert that `pyproject.toml` was not updated.
    assert_eq!(
        pyproject,
        fs::read_to_string(project_path.join("pyproject.toml")).unwrap()
    );

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
    description = "A description"
    requires-python = ">=3.13"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]

    [tool.uv]
    default-groups = [
        "dev",
        "typing",
    ]
    "###);
}

#[test]
fn test_replaces_existing_project() {
    let project_path = Path::new(FIXTURES_PATH).join("existing_project");

    assert_cmd_snapshot!(cli()
        .arg(&project_path)
        .arg("--dry-run")
        .arg("--replace-project-section"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Migrated pyproject.toml:
    [project]
    name = "foo"
    version = "0.0.1"
    description = "A description"
    requires-python = ">=3.11,<4"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]

    [tool.uv]
    default-groups = [
        "dev",
        "typing",
    ]
    "#);
}

#[test]
fn test_pep_621() {
    let project_path = Path::new(FIXTURES_PATH).join("pep_621");

    apply_filters!();
    assert_cmd_snapshot!(cli().arg(&project_path).arg("--dry-run"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Migrated pyproject.toml:
    [build-system]
    requires = ["uv_build>=[LOWER_BOUND],<[UPPER_BOUND]"]
    build-backend = "uv_build"

    [project]
    name = "foobar"
    version = "0.1.0"
    description = "A fabulous project."
    authors = [{ name = "John Doe", email = "john.doe@example.com" }]
    requires-python = ">=3.11"
    readme = "README.md"
    license = "MIT"
    maintainers = [{ name = "Dohn Joe", email = "dohn.joe@example.com" }]
    keywords = ["foo"]
    classifiers = ["Development Status :: 3 - Alpha"]
    dependencies = [
        "arrow==1.2.3",
        "git-dep",
        "private-dep==3.4.5",
    ]

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]
    profiling = ["pyinstrument>=5.0.2,<6"]

    [tool.uv]
    default-groups = [
        "dev",
        "typing",
    ]

    [[tool.uv.index]]
    name = "PyPI"
    url = "https://pypi.org/simple/"
    default = true

    [[tool.uv.index]]
    name = "supplemental"
    url = "https://supplemental.example.com/simple/"

    [tool.uv.sources]
    git-dep = { git = "https://example.com/foo/bar", tag = "v1.2.3" }
    private-dep = { index = "supplemental" }

    [tool.ruff]
    fix = true

    [tool.ruff.lint]
    # This comment should be preserved.
    fixable = ["I", "UP"]

    [tool.ruff.format]
    preview = true
    "#);
}

#[test]
fn test_manage_errors() {
    let fixture_path = Path::new(FIXTURES_PATH).join("with_migration_errors");
    let pyproject = fs::read_to_string(fixture_path.join("pyproject.toml")).unwrap();

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(fixture_path, project_path).unwrap();

    apply_filters!();
    assert_cmd_snapshot!(cli().arg(project_path), @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    error: Could not automatically migrate the project to uv because of the following errors:
    error: - Found multiple files ("README.md", "README2.md") in "tool.poetry.readme". PEP 621 only supports setting one. Make sure to manually edit the section before migrating.
    error: - "caret-or" dependency with version "^1.0||^2.0||^3.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "caret-or-single" dependency with version "^1.0|^2.0|^3.0" contains "|", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "caret-or-whitespaces" dependency with version " ^1.0 || ^2.0  ||  ^3.0 " contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "caret-or-mix-single-double-whitespaces" dependency with version " ^1.0 | ^2.0  ||  ^3.0 " contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "caret-or-and-pep-440" dependency with version "^1.0,<1.3||^2.0,<2.2" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "caret-or-table-version" dependency with version "^1.0||^2.0||^3.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "caret-or-multiple-constraints" dependency with version "^1.0||^2.0||^3.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "caret-or-multiple-constraints" dependency with version "^1.0||^2.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or" dependency with version "~1.0||~2.0||~3.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or-single" dependency with version "~1.0|~2.0|~3.0" contains "|", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or-whitespaces" dependency with version " ~1.0 || ~2.0  ||  ~3.0 " contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or-mix-single-double-whitespaces" dependency with version " ~1.0 | ~2.0  ||  ~3.0 " contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or-and-pep-440" dependency with version "~1.0,<1.1||~1.0.1,<1.0.2" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or-table-version" dependency with version "~1.0||~2.0||~3.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or-multiple-constraints" dependency with version "~1.0||~2.0||~3.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or-multiple-constraints" dependency with version "~1.0||~2.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "whitespace" dependency with version ">=7.0 <7.1" could not be transformed to PEP 440 format. Make sure to check https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#unsupported-version-specifiers.
    error: - "whitespace-multiple" dependency with version ">=7.0  <7.1" could not be transformed to PEP 440 format. Make sure to check https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#unsupported-version-specifiers.
    error: - "whitespace-caret" dependency with version "7.0 ^7.1" could not be transformed to PEP 440 format. Make sure to check https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#unsupported-version-specifiers.
    error: - "whitespace-caret-multiple" dependency with version "7.0  ^7.1" could not be transformed to PEP 440 format. Make sure to check https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#unsupported-version-specifiers.
    error: - "python-caret-or" dependency with python marker "^3.11 || ^3.12" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "python-caret-or-single" dependency with python marker "^3.11 | ^3.12" contains "|", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "python-whitespace" dependency with python marker "3.11 <=3.14" could not be transformed to PEP 440 format. Make sure to check https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#unsupported-version-specifiers.
    "#);

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
fn test_manage_warnings() {
    let fixture_path = Path::new(FIXTURES_PATH).join("with_migration_warnings");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(fixture_path, project_path).unwrap();

    apply_filters!();
    assert_cmd_snapshot!(cli().arg(project_path), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Locking dependencies with constraints from existing lock file(s) using "uv lock"...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Locking dependencies again using "uv lock" to remove constraints...
    Using [PYTHON_INTERPRETER]
    Resolved [PACKAGES] packages in [TIME]
    Successfully migrated project from Poetry to uv!

    warning: The following warnings occurred during the migration:
    warning: - Could not find dependency "non-existing-dependency" listed in "extra-with-non-existing-dependencies" extra.
    "#);

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = ">=3.11,<4"
    dependencies = ["arrow>=1.2.3,<2"]

    [project.optional-dependencies]
    extra-with-non-existing-dependencies = []

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]
    profiling = ["pyinstrument>=5.0.2,<6"]

    [tool.uv]
    package = false
    default-groups = [
        "dev",
        "typing",
    ]
    "#);

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
fn test_manage_errors_dry_run() {
    let project_path = Path::new(FIXTURES_PATH).join("with_migration_errors");
    let pyproject = fs::read_to_string(project_path.join("pyproject.toml")).unwrap();

    assert_cmd_snapshot!(cli().arg(&project_path).arg("--dry-run"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    error: Could not automatically migrate the project to uv because of the following errors:
    error: - Found multiple files ("README.md", "README2.md") in "tool.poetry.readme". PEP 621 only supports setting one. Make sure to manually edit the section before migrating.
    error: - "caret-or" dependency with version "^1.0||^2.0||^3.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "caret-or-single" dependency with version "^1.0|^2.0|^3.0" contains "|", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "caret-or-whitespaces" dependency with version " ^1.0 || ^2.0  ||  ^3.0 " contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "caret-or-mix-single-double-whitespaces" dependency with version " ^1.0 | ^2.0  ||  ^3.0 " contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "caret-or-and-pep-440" dependency with version "^1.0,<1.3||^2.0,<2.2" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "caret-or-table-version" dependency with version "^1.0||^2.0||^3.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "caret-or-multiple-constraints" dependency with version "^1.0||^2.0||^3.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "caret-or-multiple-constraints" dependency with version "^1.0||^2.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or" dependency with version "~1.0||~2.0||~3.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or-single" dependency with version "~1.0|~2.0|~3.0" contains "|", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or-whitespaces" dependency with version " ~1.0 || ~2.0  ||  ~3.0 " contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or-mix-single-double-whitespaces" dependency with version " ~1.0 | ~2.0  ||  ~3.0 " contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or-and-pep-440" dependency with version "~1.0,<1.1||~1.0.1,<1.0.2" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or-table-version" dependency with version "~1.0||~2.0||~3.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or-multiple-constraints" dependency with version "~1.0||~2.0||~3.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "tilde-or-multiple-constraints" dependency with version "~1.0||~2.0" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "whitespace" dependency with version ">=7.0 <7.1" could not be transformed to PEP 440 format. Make sure to check https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#unsupported-version-specifiers.
    error: - "whitespace-multiple" dependency with version ">=7.0  <7.1" could not be transformed to PEP 440 format. Make sure to check https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#unsupported-version-specifiers.
    error: - "whitespace-caret" dependency with version "7.0 ^7.1" could not be transformed to PEP 440 format. Make sure to check https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#unsupported-version-specifiers.
    error: - "whitespace-caret-multiple" dependency with version "7.0  ^7.1" could not be transformed to PEP 440 format. Make sure to check https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#unsupported-version-specifiers.
    error: - "python-caret-or" dependency with python marker "^3.11 || ^3.12" contains "||", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "python-caret-or-single" dependency with python marker "^3.11 | ^3.12" contains "|", which is specific to Poetry and not supported by PEP 440. See https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#operator for guidance.
    error: - "python-whitespace" dependency with python marker "3.11 <=3.14" could not be transformed to PEP 440 format. Make sure to check https://mkniewallner.github.io/migrate-to-uv/supported-package-managers/#unsupported-version-specifiers.
    "#);

    // Assert that `pyproject.toml` was not updated.
    assert_eq!(
        pyproject,
        fs::read_to_string(project_path.join("pyproject.toml")).unwrap()
    );

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}

#[test]
fn test_manage_warnings_dry_run() {
    let project_path = Path::new(FIXTURES_PATH).join("with_migration_warnings");
    let pyproject = fs::read_to_string(project_path.join("pyproject.toml")).unwrap();

    assert_cmd_snapshot!(cli().arg(&project_path).arg("--dry-run"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Migrated pyproject.toml:
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = ">=3.11,<4"
    dependencies = ["arrow>=1.2.3,<2"]

    [project.optional-dependencies]
    extra-with-non-existing-dependencies = []

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]
    profiling = ["pyinstrument>=5.0.2,<6"]

    [tool.uv]
    package = false
    default-groups = [
        "dev",
        "typing",
    ]

    warning: The following warnings occurred during the migration:
    warning: - Could not find dependency "non-existing-dependency" listed in "extra-with-non-existing-dependencies" extra.
    "#);

    // Assert that `pyproject.toml` was not updated.
    assert_eq!(
        pyproject,
        fs::read_to_string(project_path.join("pyproject.toml")).unwrap()
    );

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}

#[test]
fn test_build_backend_auto_hatch() {
    let fixture_path = Path::new(FIXTURES_PATH).join("build_backend_hatch");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(&fixture_path, project_path).unwrap();

    Command::new("uvx")
        .arg("poetry")
        .arg("build")
        .current_dir(project_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    let sdist_files_before = get_tar_gz_entries(&project_path.join("dist"), "foobar-0.1.0.tar.gz");
    let wheel_files_before =
        get_zip_entries(&project_path.join("dist"), "foobar-0.1.0-py3-none-any.whl");

    remove_dir_all(project_path.join("dist")).unwrap();

    assert_cmd_snapshot!(cli().arg(project_path).arg("--skip-lock"), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Successfully migrated project from Poetry to uv!

    warning: The following warnings occurred during the migration:
    warning: - Migrating build backend to Hatch because package distribution metadata is too complex for uv.
    warning: - Build backend was migrated to Hatch. It is highly recommended to manually check that files included in the source distribution and wheels are the same than before the migration.
    ");

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [build-system]
    requires = ["hatchling"]
    build-backend = "hatchling.build"

    [project]
    name = "foobar"
    version = "0.1.0"
    description = "A fabulous project."
    authors = [{ name = "John Doe", email = "john.doe@example.com" }]
    requires-python = ">=3.10"
    classifiers = [
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Python :: 3.13",
        "Programming Language :: Python :: 3.14",
    ]

    [tool.hatch.build.targets.sdist]
    include = [
        "packages_sdist_wheel",
        "packages_sdist_wheel_2",
        "packages_sdist",
        "packages_sdist_2",
        "packages_glob_sdist_wheel/**/*.py",
        "packages_glob_sdist_wheel_2/**/*.py",
        "packages_glob_sdist/**/*.py",
        "packages_glob_sdist_2/**/*.py",
        "from/packages_from_sdist_wheel",
        "packages_to_sdist_wheel",
        "from/packages_from_to_sdist_wheel",
        "packages_sdist_wheel_with_excluded_files",
        "text_file_sdist_wheel.txt",
        "text_file_sdist.txt",
    ]
    exclude = [
        "packages_sdist_wheel_with_excluded_files/bar.py",
        "packages_sdist_wheel_with_excluded_files/foobar",
    ]

    [tool.hatch.build.targets.sdist.force-include]
    include_sdist = "include_sdist"
    include_sdist_2 = "include_sdist_2"
    include_sdist_3 = "include_sdist_3"
    include_sdist_4 = "include_sdist_4"
    include_sdist_wheel = "include_sdist_wheel"

    [tool.hatch.build.targets.wheel]
    include = [
        "packages_sdist_wheel",
        "packages_sdist_wheel_2",
        "packages_wheel",
        "packages_wheel_2",
        "packages_glob_sdist_wheel/**/*.py",
        "packages_glob_sdist_wheel_2/**/*.py",
        "packages_glob_wheel/**/*.py",
        "packages_glob_wheel_2/**/*.py",
        "from/packages_from_sdist_wheel",
        "packages_to_sdist_wheel",
        "from/packages_from_to_sdist_wheel",
        "packages_sdist_wheel_with_excluded_files",
        "text_file_sdist_wheel.txt",
        "text_file_wheel.txt",
    ]
    exclude = [
        "packages_sdist_wheel_with_excluded_files/bar.py",
        "packages_sdist_wheel_with_excluded_files/foobar",
    ]

    [tool.hatch.build.targets.wheel.force-include]
    include_sdist_wheel = "include_sdist_wheel"
    include_wheel = "include_wheel"
    include_wheel_2 = "include_wheel_2"

    [tool.hatch.build.targets.wheel.sources]
    "from/packages_from_sdist_wheel" = "packages_from_sdist_wheel"
    packages_to_sdist_wheel = "to/packages_to_sdist_wheel"
    "from/packages_from_to_sdist_wheel" = "to/packages_from_to_sdist_wheel"
    "#);

    Command::new("uvx")
        .arg("hatch")
        .arg("build")
        .current_dir(project_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    let sdist_files_after = get_tar_gz_entries(&project_path.join("dist"), "foobar-0.1.0.tar.gz");
    let wheel_files_after =
        get_zip_entries(&project_path.join("dist"), "foobar-0.1.0-py3-none-any.whl");

    assert_eq!(sdist_files_before, sdist_files_after);
    assert_eq!(wheel_files_before, wheel_files_after);
}

#[test]
fn test_build_backend_auto_uv() {
    let fixture_path = Path::new(FIXTURES_PATH).join("build_backend_uv");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(&fixture_path, project_path).unwrap();

    Command::new("uvx")
        .arg("poetry")
        .arg("build")
        .current_dir(project_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    let sdist_files_before = get_tar_gz_entries(&project_path.join("dist"), "foobar-0.1.0.tar.gz");
    let wheel_files_before =
        get_zip_entries(&project_path.join("dist"), "foobar-0.1.0-py3-none-any.whl");

    remove_dir_all(project_path.join("dist")).unwrap();

    assert_cmd_snapshot!(cli().arg(project_path).arg("--skip-lock"), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Successfully migrated project from Poetry to uv!

    warning: The following warnings occurred during the migration:
    warning: - Build backend was migrated to uv. It is highly recommended to manually check that files included in the source distribution and wheels are the same than before the migration.
    ");

    apply_filters!();
    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [build-system]
    requires = ["uv_build>=[LOWER_BOUND],<[UPPER_BOUND]"]
    build-backend = "uv_build"

    [project]
    name = "foobar"
    version = "0.1.0"
    description = "A fabulous project."
    authors = [{ name = "John Doe", email = "john.doe@example.com" }]
    requires-python = ">=3.10"
    classifiers = [
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Python :: 3.13",
        "Programming Language :: Python :: 3.14",
    ]

    [tool.uv.build-backend]
    module-name = [
        "packages_sdist_wheel",
        "packages_sdist_wheel_2",
        "packages_sdist",
        "packages_sdist_2",
        "packages_sdist_wheel_with_excluded_files",
    ]
    module-root = ""
    source-exclude = [
        "packages_sdist_wheel_with_excluded_files/bar.py",
        "packages_sdist_wheel_with_excluded_files/foobar",
    ]
    source-include = [
        "packages_glob_sdist/**/*.py",
        "packages_glob_sdist_2/**/*.py",
        "text_file_sdist.txt",
        "FILE_WITHOUT_EXTENSION_SDIST",
        "include_sdist/**",
        "include_sdist_2/**",
        "include_sdist_3/**",
        "include_sdist_4/**",
        "INCLUDE_FILE_WITHOUT_EXTENSION_SDIST",
    ]
    wheel-exclude = [
        "packages_sdist",
        "packages_sdist_2",
        "packages_sdist_wheel_with_excluded_files/bar.py",
        "packages_sdist_wheel_with_excluded_files/foobar",
    ]
    "#);

    Command::new("uv")
        .arg("build")
        .current_dir(project_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    let sdist_files_after = get_tar_gz_entries(&project_path.join("dist"), "foobar-0.1.0.tar.gz");
    let wheel_files_after =
        get_zip_entries(&project_path.join("dist"), "foobar-0.1.0-py3-none-any.whl");

    assert_eq!(sdist_files_before, sdist_files_after);
    assert_eq!(wheel_files_before, wheel_files_after);
}

#[test]
fn test_build_backend_auto_keep_current_build_backend() {
    let fixture_path = Path::new(FIXTURES_PATH).join("build_backend_hatch");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(&fixture_path, project_path).unwrap();

    Command::new("uvx")
        .arg("poetry")
        .arg("build")
        .current_dir(project_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    let sdist_files_before = get_tar_gz_entries(&project_path.join("dist"), "foobar-0.1.0.tar.gz");
    let wheel_files_before =
        get_zip_entries(&project_path.join("dist"), "foobar-0.1.0-py3-none-any.whl");

    remove_dir_all(project_path.join("dist")).unwrap();

    assert_cmd_snapshot!(cli().arg(project_path).arg("--skip-lock").arg("--keep-current-build-backend"), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Successfully migrated project from Poetry to uv!
    ");

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [project]
    name = "foobar"
    version = "0.1.0"
    description = "A fabulous project."
    authors = [{ name = "John Doe", email = "john.doe@example.com" }]
    requires-python = ">=3.10"

    [build-system]
    requires = ["poetry-core>=1.0.0"]
    build-backend = "poetry.core.masonry.api"

    [tool.poetry]
    packages = [
        { include = "packages_sdist_wheel" },
        { include = "packages_sdist_wheel_2", format = ["sdist", "wheel"] },
        { include = "packages_sdist", format = "sdist" },
        { include = "packages_sdist_2", format = ["sdist"] },
        { include = "packages_wheel", format = "wheel" },
        { include = "packages_wheel_2", format = ["wheel"] },
        # An empty array for `format` means that files are not included anywhere.
        { include = "packages_nowhere", format = [] },
        { include = "packages_glob_sdist_wheel/**/*.py" },
        { include = "packages_glob_sdist_wheel_2/**/*.py", format = ["sdist", "wheel"] },
        { include = "packages_glob_sdist/**/*.py", format = "sdist" },
        { include = "packages_glob_sdist_2/**/*.py", format = ["sdist"] },
        { include = "packages_glob_wheel/**/*.py", format = "wheel" },
        { include = "packages_glob_wheel_2/**/*.py", format = ["wheel"] },
        # An empty array for `format` means that files are not included anywhere.
        { include = "packages_glob_nowhere/**/*.py", format = [] },
        { include = "packages_from_sdist_wheel", from = "from" },
        { include = "packages_to_sdist_wheel", to = "to" },
        { include = "packages_from_to_sdist_wheel", from = "from", to = "to" },
        { include = "packages_sdist_wheel_with_excluded_files" },
        { include = "text_file_sdist_wheel.txt" },
        { include = "text_file_sdist.txt", format = "sdist" },
        { include = "text_file_wheel.txt", format = "wheel" },
    ]
    include = [
        "include_sdist",
        { path = "include_sdist_2" },
        { path = "include_sdist_3", format = "sdist" },
        { path = "include_sdist_4", format = ["sdist"] },
        { path = "include_sdist_wheel", format = ["sdist", "wheel"] },
        { path = "include_wheel", format = "wheel" },
        { path = "include_wheel_2", format = ["wheel"] },
        # An empty array for `format` means that files are not included anywhere.
        { path = "include_nowhere", format = [] },
    ]
    exclude = [
        "packages_sdist_wheel_with_excluded_files/bar.py",
        "packages_sdist_wheel_with_excluded_files/foobar",
    ]
    "#);

    Command::new("uvx")
        .arg("poetry")
        .arg("build")
        .current_dir(project_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    let sdist_files_after = get_tar_gz_entries(&project_path.join("dist"), "foobar-0.1.0.tar.gz");
    let wheel_files_after =
        get_zip_entries(&project_path.join("dist"), "foobar-0.1.0-py3-none-any.whl");

    assert_eq!(sdist_files_before, sdist_files_after);
    assert_eq!(wheel_files_before, wheel_files_after);
}

#[test]
fn test_build_backend_auto_errors() {
    let fixture_path = Path::new(FIXTURES_PATH).join("build_backend_hatch_incompatible");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(&fixture_path, project_path).unwrap();

    // Ensure that the project is valid for Poetry, even if we cannot convert it to uv.
    Command::new("uvx")
        .arg("poetry")
        .arg("build")
        .current_dir(project_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    assert_cmd_snapshot!(cli().arg(project_path), @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    error: Could not automatically migrate the project to uv because of the following errors:
    error: - "foo.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" on a file, which cannot be expressed with Hatch.
    error: - "bar.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "to" on a file, which cannot be expressed with Hatch.
    error: - "foobar.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" and "to" on a file, which cannot be expressed with Hatch.
    error: - "a_directory/foo.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" and "to" on a file, which cannot be expressed with Hatch.
    error: - "a_directory/another_directory/foo.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" and "to" on a file, which cannot be expressed with Hatch.
    error: - "packages_glob_from/**/*.py" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from", which cannot be expressed with Hatch.
    error: - "packages_glob_to/**/*.py" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "to", which cannot be expressed with Hatch.
    error: - "packages_glob_from_to/**/*.py" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from" and "to", which cannot be expressed with Hatch.
    error: - "**/*.json" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from", which cannot be expressed with Hatch.
    error: - "**/*.json" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "to", which cannot be expressed with Hatch.
    error: - "**/*.yaml" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from" and "to", which cannot be expressed with Hatch.
    error: - Package distribution could not be migrated to uv nor Hatch build backend due to the issues above. Consider keeping the current build backend with "--keep-current-build-backend".
    "#);
}

#[test]
fn test_build_backend_auto_errors_dry_run() {
    let project_path = Path::new(FIXTURES_PATH).join("build_backend_hatch_incompatible");
    let pyproject = fs::read_to_string(project_path.join("pyproject.toml")).unwrap();

    assert_cmd_snapshot!(cli().arg(&project_path).arg("--dry-run"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    error: Could not automatically migrate the project to uv because of the following errors:
    error: - "foo.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" on a file, which cannot be expressed with Hatch.
    error: - "bar.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "to" on a file, which cannot be expressed with Hatch.
    error: - "foobar.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" and "to" on a file, which cannot be expressed with Hatch.
    error: - "a_directory/foo.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" and "to" on a file, which cannot be expressed with Hatch.
    error: - "a_directory/another_directory/foo.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" and "to" on a file, which cannot be expressed with Hatch.
    error: - "packages_glob_from/**/*.py" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from", which cannot be expressed with Hatch.
    error: - "packages_glob_to/**/*.py" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "to", which cannot be expressed with Hatch.
    error: - "packages_glob_from_to/**/*.py" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from" and "to", which cannot be expressed with Hatch.
    error: - "**/*.json" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from", which cannot be expressed with Hatch.
    error: - "**/*.json" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "to", which cannot be expressed with Hatch.
    error: - "**/*.yaml" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from" and "to", which cannot be expressed with Hatch.
    error: - Package distribution could not be migrated to uv nor Hatch build backend due to the issues above. Consider keeping the current build backend with "--keep-current-build-backend".
    "#);

    // Assert that `pyproject.toml` was not updated.
    assert_eq!(
        pyproject,
        fs::read_to_string(project_path.join("pyproject.toml")).unwrap()
    );
}

#[test]
fn test_build_backend_hatch() {
    let fixture_path = Path::new(FIXTURES_PATH).join("build_backend_hatch");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(&fixture_path, project_path).unwrap();

    Command::new("uvx")
        .arg("poetry")
        .arg("build")
        .current_dir(project_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    let sdist_files_before = get_tar_gz_entries(&project_path.join("dist"), "foobar-0.1.0.tar.gz");
    let wheel_files_before =
        get_zip_entries(&project_path.join("dist"), "foobar-0.1.0-py3-none-any.whl");

    remove_dir_all(project_path.join("dist")).unwrap();

    assert_cmd_snapshot!(cli().arg(project_path).arg("--skip-lock").arg("--build-backend").arg("hatch"), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Successfully migrated project from Poetry to uv!

    warning: The following warnings occurred during the migration:
    warning: - Build backend was migrated to Hatch. It is highly recommended to manually check that files included in the source distribution and wheels are the same than before the migration.
    ");

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [build-system]
    requires = ["hatchling"]
    build-backend = "hatchling.build"

    [project]
    name = "foobar"
    version = "0.1.0"
    description = "A fabulous project."
    authors = [{ name = "John Doe", email = "john.doe@example.com" }]
    requires-python = ">=3.10"
    classifiers = [
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Python :: 3.13",
        "Programming Language :: Python :: 3.14",
    ]

    [tool.hatch.build.targets.sdist]
    include = [
        "packages_sdist_wheel",
        "packages_sdist_wheel_2",
        "packages_sdist",
        "packages_sdist_2",
        "packages_glob_sdist_wheel/**/*.py",
        "packages_glob_sdist_wheel_2/**/*.py",
        "packages_glob_sdist/**/*.py",
        "packages_glob_sdist_2/**/*.py",
        "from/packages_from_sdist_wheel",
        "packages_to_sdist_wheel",
        "from/packages_from_to_sdist_wheel",
        "packages_sdist_wheel_with_excluded_files",
        "text_file_sdist_wheel.txt",
        "text_file_sdist.txt",
    ]
    exclude = [
        "packages_sdist_wheel_with_excluded_files/bar.py",
        "packages_sdist_wheel_with_excluded_files/foobar",
    ]

    [tool.hatch.build.targets.sdist.force-include]
    include_sdist = "include_sdist"
    include_sdist_2 = "include_sdist_2"
    include_sdist_3 = "include_sdist_3"
    include_sdist_4 = "include_sdist_4"
    include_sdist_wheel = "include_sdist_wheel"

    [tool.hatch.build.targets.wheel]
    include = [
        "packages_sdist_wheel",
        "packages_sdist_wheel_2",
        "packages_wheel",
        "packages_wheel_2",
        "packages_glob_sdist_wheel/**/*.py",
        "packages_glob_sdist_wheel_2/**/*.py",
        "packages_glob_wheel/**/*.py",
        "packages_glob_wheel_2/**/*.py",
        "from/packages_from_sdist_wheel",
        "packages_to_sdist_wheel",
        "from/packages_from_to_sdist_wheel",
        "packages_sdist_wheel_with_excluded_files",
        "text_file_sdist_wheel.txt",
        "text_file_wheel.txt",
    ]
    exclude = [
        "packages_sdist_wheel_with_excluded_files/bar.py",
        "packages_sdist_wheel_with_excluded_files/foobar",
    ]

    [tool.hatch.build.targets.wheel.force-include]
    include_sdist_wheel = "include_sdist_wheel"
    include_wheel = "include_wheel"
    include_wheel_2 = "include_wheel_2"

    [tool.hatch.build.targets.wheel.sources]
    "from/packages_from_sdist_wheel" = "packages_from_sdist_wheel"
    packages_to_sdist_wheel = "to/packages_to_sdist_wheel"
    "from/packages_from_to_sdist_wheel" = "to/packages_from_to_sdist_wheel"
    "#);

    Command::new("uvx")
        .arg("hatch")
        .arg("build")
        .current_dir(project_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    let sdist_files_after = get_tar_gz_entries(&project_path.join("dist"), "foobar-0.1.0.tar.gz");
    let wheel_files_after =
        get_zip_entries(&project_path.join("dist"), "foobar-0.1.0-py3-none-any.whl");

    assert_eq!(sdist_files_before, sdist_files_after);
    assert_eq!(wheel_files_before, wheel_files_after);
}

#[test]
fn test_build_backend_hatch_errors() {
    let fixture_path = Path::new(FIXTURES_PATH).join("build_backend_hatch_incompatible");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(&fixture_path, project_path).unwrap();

    // Ensure that the project is valid for Poetry, even if we cannot convert it to uv.
    Command::new("uvx")
        .arg("poetry")
        .arg("build")
        .current_dir(project_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    assert_cmd_snapshot!(cli().arg(project_path).arg("--build-backend").arg("hatch"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    error: Could not automatically migrate the project to uv because of the following errors:
    error: - "foo.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" on a file, which cannot be expressed with Hatch.
    error: - "bar.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "to" on a file, which cannot be expressed with Hatch.
    error: - "foobar.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" and "to" on a file, which cannot be expressed with Hatch.
    error: - "a_directory/foo.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" and "to" on a file, which cannot be expressed with Hatch.
    error: - "a_directory/another_directory/foo.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" and "to" on a file, which cannot be expressed with Hatch.
    error: - "packages_glob_from/**/*.py" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from", which cannot be expressed with Hatch.
    error: - "packages_glob_to/**/*.py" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "to", which cannot be expressed with Hatch.
    error: - "packages_glob_from_to/**/*.py" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from" and "to", which cannot be expressed with Hatch.
    error: - "**/*.json" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from", which cannot be expressed with Hatch.
    error: - "**/*.json" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "to", which cannot be expressed with Hatch.
    error: - "**/*.yaml" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from" and "to", which cannot be expressed with Hatch.
    error: - Package distribution could not be migrated to Hatch build backend due to the issues above. Consider keeping the current build backend with "--keep-current-build-backend".
    "#);
}

#[test]
fn test_build_backend_hatch_errors_dry_run() {
    let project_path = Path::new(FIXTURES_PATH).join("build_backend_hatch_incompatible");
    let pyproject = fs::read_to_string(project_path.join("pyproject.toml")).unwrap();

    assert_cmd_snapshot!(cli().arg(&project_path).arg("--dry-run").arg("--build-backend").arg("hatch"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    error: Could not automatically migrate the project to uv because of the following errors:
    error: - "foo.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" on a file, which cannot be expressed with Hatch.
    error: - "bar.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "to" on a file, which cannot be expressed with Hatch.
    error: - "foobar.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" and "to" on a file, which cannot be expressed with Hatch.
    error: - "a_directory/foo.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" and "to" on a file, which cannot be expressed with Hatch.
    error: - "a_directory/another_directory/foo.txt" from "poetry.packages.include" cannot be converted to Hatch, as it uses "from" and "to" on a file, which cannot be expressed with Hatch.
    error: - "packages_glob_from/**/*.py" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from", which cannot be expressed with Hatch.
    error: - "packages_glob_to/**/*.py" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "to", which cannot be expressed with Hatch.
    error: - "packages_glob_from_to/**/*.py" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from" and "to", which cannot be expressed with Hatch.
    error: - "**/*.json" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from", which cannot be expressed with Hatch.
    error: - "**/*.json" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "to", which cannot be expressed with Hatch.
    error: - "**/*.yaml" from "poetry.packages.include" cannot be converted to Hatch, as it uses glob pattern with "from" and "to", which cannot be expressed with Hatch.
    error: - Package distribution could not be migrated to Hatch build backend due to the issues above. Consider keeping the current build backend with "--keep-current-build-backend".
    "#);

    // Assert that `pyproject.toml` was not updated.
    assert_eq!(
        pyproject,
        fs::read_to_string(project_path.join("pyproject.toml")).unwrap()
    );
}

#[test]
fn test_build_backend_uv() {
    let fixture_path = Path::new(FIXTURES_PATH).join("build_backend_uv");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(&fixture_path, project_path).unwrap();

    Command::new("uvx")
        .arg("poetry")
        .arg("build")
        .current_dir(project_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    let sdist_files_before = get_tar_gz_entries(&project_path.join("dist"), "foobar-0.1.0.tar.gz");
    let wheel_files_before =
        get_zip_entries(&project_path.join("dist"), "foobar-0.1.0-py3-none-any.whl");

    remove_dir_all(project_path.join("dist")).unwrap();

    assert_cmd_snapshot!(cli().arg(project_path).arg("--skip-lock").arg("--build-backend").arg("uv"), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Successfully migrated project from Poetry to uv!

    warning: The following warnings occurred during the migration:
    warning: - Build backend was migrated to uv. It is highly recommended to manually check that files included in the source distribution and wheels are the same than before the migration.
    ");

    apply_filters!();
    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r#"
    [build-system]
    requires = ["uv_build>=[LOWER_BOUND],<[UPPER_BOUND]"]
    build-backend = "uv_build"

    [project]
    name = "foobar"
    version = "0.1.0"
    description = "A fabulous project."
    authors = [{ name = "John Doe", email = "john.doe@example.com" }]
    requires-python = ">=3.10"
    classifiers = [
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Python :: 3.13",
        "Programming Language :: Python :: 3.14",
    ]

    [tool.uv.build-backend]
    module-name = [
        "packages_sdist_wheel",
        "packages_sdist_wheel_2",
        "packages_sdist",
        "packages_sdist_2",
        "packages_sdist_wheel_with_excluded_files",
    ]
    module-root = ""
    source-exclude = [
        "packages_sdist_wheel_with_excluded_files/bar.py",
        "packages_sdist_wheel_with_excluded_files/foobar",
    ]
    source-include = [
        "packages_glob_sdist/**/*.py",
        "packages_glob_sdist_2/**/*.py",
        "text_file_sdist.txt",
        "FILE_WITHOUT_EXTENSION_SDIST",
        "include_sdist/**",
        "include_sdist_2/**",
        "include_sdist_3/**",
        "include_sdist_4/**",
        "INCLUDE_FILE_WITHOUT_EXTENSION_SDIST",
    ]
    wheel-exclude = [
        "packages_sdist",
        "packages_sdist_2",
        "packages_sdist_wheel_with_excluded_files/bar.py",
        "packages_sdist_wheel_with_excluded_files/foobar",
    ]
    "#);

    Command::new("uv")
        .arg("build")
        .current_dir(project_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    let sdist_files_after = get_tar_gz_entries(&project_path.join("dist"), "foobar-0.1.0.tar.gz");
    let wheel_files_after =
        get_zip_entries(&project_path.join("dist"), "foobar-0.1.0-py3-none-any.whl");

    assert_eq!(sdist_files_before, sdist_files_after);
    assert_eq!(wheel_files_before, wheel_files_after);
}

#[test]
fn test_build_backend_uv_errors() {
    let fixture_path = Path::new(FIXTURES_PATH).join("build_backend_uv_incompatible");

    let tmp_dir = tempdir().unwrap();
    let project_path = tmp_dir.path();

    copy_dir(&fixture_path, project_path).unwrap();

    // Ensure that the project is valid for Poetry, even if we cannot convert it to uv.
    Command::new("uvx")
        .arg("poetry")
        .arg("build")
        .current_dir(project_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    assert_cmd_snapshot!(cli().arg(project_path).arg("--build-backend").arg("uv"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    error: Could not automatically migrate the project to uv because of the following errors:
    error: - "packages_wheel" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to wheels only, which cannot be expressed with uv.
    error: - "packages_wheel_2" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to wheels only, which cannot be expressed with uv.
    error: - "packages_glob_sdist_wheel/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, and uses globs, which cannot be expressed with uv.
    error: - "packages_glob_sdist_wheel_2/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, and uses globs, which cannot be expressed with uv.
    error: - "packages_glob_wheel/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to wheels only, and uses globs, which cannot be expressed with uv.
    error: - "packages_glob_wheel_2/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to wheels only, and uses globs, which cannot be expressed with uv.
    error: - "packages_from_sdist_wheel" from "poetry.packages.include" cannot be converted to uv, as it uses "from", which cannot be expressed with uv.
    error: - "packages_to_sdist_wheel" from "poetry.packages.include" cannot be converted to uv, as it uses "to", which cannot be expressed with uv.
    error: - "packages_from_to_sdist_wheel" from "poetry.packages.include" cannot be converted to uv, as it uses "from", which cannot be expressed with uv.
    error: - "packages_from_to_sdist_wheel" from "poetry.packages.include" cannot be converted to uv, as it uses "to", which cannot be expressed with uv.
    error: - "packages_glob_to_sdist_wheel/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it uses "to", which cannot be expressed with uv.
    error: - "packages_glob_to_sdist_wheel/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, and uses globs, which cannot be expressed with uv.
    error: - "packages_glob_from_to_sdist_wheel/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it uses "from", which cannot be expressed with uv.
    error: - "packages_glob_from_to_sdist_wheel/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it uses "to", which cannot be expressed with uv.
    error: - "packages_glob_from_to_sdist_wheel/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, and uses globs, which cannot be expressed with uv.
    error: - "text_file_sdist_wheel.txt" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, and is a file, which cannot be expressed with uv.
    error: - "text_file_wheel.txt" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to wheels only, and is a file, which cannot be expressed with uv.
    error: - "packages_without_init" from "poetry.packages.include" cannot be converted to uv, as it does not contain an "__init__.py" file, which is required by uv for packages.
    error: - "packages_without_init_root" from "poetry.packages.include" cannot be converted to uv, as it does not contain an "__init__.py" file, which is required by uv for packages.
    error: - "packages_from_without_init" from "poetry.packages.include" cannot be converted to uv, as it uses "from", which cannot be expressed with uv.
    error: - "packages_from_without_init" from "poetry.packages.include" cannot be converted to uv, as it does not contain an "__init__.py" file, which is required by uv for packages.
    error: - "packages_from_without_init_root" from "poetry.packages.include" cannot be converted to uv, as it uses "from", which cannot be expressed with uv.
    error: - "packages_from_without_init_root" from "poetry.packages.include" cannot be converted to uv, as it does not contain an "__init__.py" file, which is required by uv for packages.
    error: - "include_sdist_wheel" from "poetry.include" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, which cannot be expressed with uv.
    error: - "include_wheel" from "poetry.include" cannot be converted to uv, as it is configured to be added to wheels only, which cannot be expressed with uv.
    error: - "include_wheel_2" from "poetry.include" cannot be converted to uv, as it is configured to be added to wheels only, which cannot be expressed with uv.
    error: - Package distribution could not be migrated to uv build backend due to the issues above. Consider using Hatch build backend with "--build-backend hatch".
    "#);
}

#[test]
fn test_build_backend_uv_errors_dry_run() {
    let project_path = Path::new(FIXTURES_PATH).join("build_backend_uv_incompatible");
    let pyproject = fs::read_to_string(project_path.join("pyproject.toml")).unwrap();

    assert_cmd_snapshot!(cli().arg(&project_path).arg("--dry-run").arg("--build-backend").arg("uv"), @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    error: Could not automatically migrate the project to uv because of the following errors:
    error: - "packages_wheel" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to wheels only, which cannot be expressed with uv.
    error: - "packages_wheel_2" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to wheels only, which cannot be expressed with uv.
    error: - "packages_glob_sdist_wheel/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, and uses globs, which cannot be expressed with uv.
    error: - "packages_glob_sdist_wheel_2/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, and uses globs, which cannot be expressed with uv.
    error: - "packages_glob_wheel/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to wheels only, and uses globs, which cannot be expressed with uv.
    error: - "packages_glob_wheel_2/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to wheels only, and uses globs, which cannot be expressed with uv.
    error: - "packages_from_sdist_wheel" from "poetry.packages.include" cannot be converted to uv, as it uses "from", which cannot be expressed with uv.
    error: - "packages_to_sdist_wheel" from "poetry.packages.include" cannot be converted to uv, as it uses "to", which cannot be expressed with uv.
    error: - "packages_from_to_sdist_wheel" from "poetry.packages.include" cannot be converted to uv, as it uses "from", which cannot be expressed with uv.
    error: - "packages_from_to_sdist_wheel" from "poetry.packages.include" cannot be converted to uv, as it uses "to", which cannot be expressed with uv.
    error: - "packages_glob_to_sdist_wheel/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it uses "to", which cannot be expressed with uv.
    error: - "packages_glob_to_sdist_wheel/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, and uses globs, which cannot be expressed with uv.
    error: - "packages_glob_from_to_sdist_wheel/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it uses "from", which cannot be expressed with uv.
    error: - "packages_glob_from_to_sdist_wheel/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it uses "to", which cannot be expressed with uv.
    error: - "packages_glob_from_to_sdist_wheel/**/*.py" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, and uses globs, which cannot be expressed with uv.
    error: - "text_file_sdist_wheel.txt" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, and is a file, which cannot be expressed with uv.
    error: - "text_file_wheel.txt" from "poetry.packages.include" cannot be converted to uv, as it is configured to be added to wheels only, and is a file, which cannot be expressed with uv.
    error: - "packages_without_init" from "poetry.packages.include" cannot be converted to uv, as it does not contain an "__init__.py" file, which is required by uv for packages.
    error: - "packages_without_init_root" from "poetry.packages.include" cannot be converted to uv, as it does not contain an "__init__.py" file, which is required by uv for packages.
    error: - "packages_from_without_init" from "poetry.packages.include" cannot be converted to uv, as it uses "from", which cannot be expressed with uv.
    error: - "packages_from_without_init" from "poetry.packages.include" cannot be converted to uv, as it does not contain an "__init__.py" file, which is required by uv for packages.
    error: - "packages_from_without_init_root" from "poetry.packages.include" cannot be converted to uv, as it uses "from", which cannot be expressed with uv.
    error: - "packages_from_without_init_root" from "poetry.packages.include" cannot be converted to uv, as it does not contain an "__init__.py" file, which is required by uv for packages.
    error: - "include_sdist_wheel" from "poetry.include" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, which cannot be expressed with uv.
    error: - "include_wheel" from "poetry.include" cannot be converted to uv, as it is configured to be added to wheels only, which cannot be expressed with uv.
    error: - "include_wheel_2" from "poetry.include" cannot be converted to uv, as it is configured to be added to wheels only, which cannot be expressed with uv.
    error: - Package distribution could not be migrated to uv build backend due to the issues above. Consider using Hatch build backend with "--build-backend hatch".
    "#);

    // Assert that `pyproject.toml` was not updated.
    assert_eq!(
        pyproject,
        fs::read_to_string(project_path.join("pyproject.toml")).unwrap()
    );

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}
