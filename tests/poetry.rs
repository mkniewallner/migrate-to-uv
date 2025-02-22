use crate::common::{LockedPackage, UvLock, apply_lock_filters, cli};
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

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r###"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = "~=3.11"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]

    [tool.uv]
    package = false
    default-groups = [
        "dev",
        "typing",
    ]
    "###);

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

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r###"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = "~=3.11"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]

    [tool.uv]
    package = false
    default-groups = [
        "dev",
        "typing",
    ]
    "###);

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

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r###"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = "~=3.11"
    dependencies = ["arrow>=1.2.3,<2"]

    [tool.poetry]
    package-mode = false
    name = "foo"

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]

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
    "###);

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

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r###"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = "~=3.11"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = [
        "factory-boy>=3.2.1,<4",
        { include = "typing" },
    ]
    typing = ["mypy>=1.13.0,<2"]

    [tool.uv]
    package = false
    "###);

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

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r###"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = "~=3.11"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]

    [tool.uv]
    package = false
    "###);

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

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r###"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = "~=3.11"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = [
        "factory-boy>=3.2.1,<4",
        "mypy>=1.13.0,<2",
    ]

    [tool.uv]
    package = false
    "###);

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

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r###"
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = "~=3.11"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]

    [tool.uv]
    package = false
    default-groups = [
        "dev",
        "typing",
    ]
    "###);

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

    insta::assert_snapshot!(fs::read_to_string(project_path.join("pyproject.toml")).unwrap(), @r###"
    [build-system]
    requires = ["hatchling"]
    build-backend = "hatchling.build"

    [project]
    name = "foobar"
    version = "0.1.0"
    description = "A fabulous project."
    authors = [{ name = "John Doe", email = "john.doe@example.com" }]
    requires-python = "~=3.11"
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
    ]
    dependencies = [
        "caret>=1.2.3,<2",
        "caret-2~=1.2",
        "caret-3>=1,<2",
        "caret-4>=0.2.3,<0.3",
        "caret-5>=0.0.3,<0.0.4",
        "caret-6>=0.0,<0.1",
        "caret-7>=0,<1",
        "caret-8>=1.2.3.4,<2",
        "caret-9>=0.1.2.3,<0.2",
        "caret-pre-release>=1.2.3b1,<2",
        "tilde~=1.2.3",
        "tilde-2>=1.2,<1.3",
        "tilde-3>=1,<2",
        "tilde-4~=1.2.3.4",
        "tilde-pre-release~=1.2.3b1",
        "exact==1.2.3",
        "star",
        "star-2==1.*",
        "star-3==1.2.*",
        "pep440>=1.2.3",
        "with-version-only==1.2.3",
        "with-extras[asyncio, postgresql_asyncpg]==1.2.3",
        "with-markers==1.2.3 ; python_version <= '3.11' or sys_platform == 'win32'",
        "with-platform==1.2.3 ; sys_platform == 'darwin'",
        "with-markers-python-platform==1.2.3 ; python_version >= '3.11' and python_version < '3.12' and platform_python_implementation == 'CPython' or platform_python_implementation == 'Jython' and sys_platform == 'darwin'",
        "with-source==1.2.3",
        "python-restricted==1.2.3 ; python_version ~= '3.11'",
        "python-restricted-2==1.2.3 ; python_version >= '3.11' and python_version < '3.12'",
        "python-restricted-3==1.2.3 ; python_version > '3.11'",
        "python-restricted-4==1.2.3 ; python_version >= '3.11'",
        "python-restricted-5==1.2.3 ; python_version < '3.11'",
        "python-restricted-6==1.2.3 ; python_version <= '3.11'",
        "python-restricted-7==1.2.3 ; python_version > '3.11' and python_version < '3.13'",
        "python-restricted-with-source==1.2.3 ; python_version > '3.11' and python_version < '3.13'",
        "whitespaces~=3.2",
        "whitespaces-2>   3.11,     <=     3.13",
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
        "multiple-constraints-python-version>=2 ; python_version >= '3.11'",
        "multiple-constraints-python-version<2 ; python_version < '3.11'",
        "multiple-constraints-platform-version>=2 ; sys_platform == 'darwin'",
        "multiple-constraints-platform-version<2 ; sys_platform == 'linux'",
        "multiple-constraints-markers-version>=2 ; platform_python_implementation == 'CPython'",
        "multiple-constraints-markers-version<2 ; platform_python_implementation != 'CPython'",
        "multiple-constraints-python-platform-markers-version>=2 ; python_version >= '3.11' and platform_python_implementation == 'CPython' and sys_platform == 'darwin'",
        "multiple-constraints-python-platform-markers-version<2 ; python_version < '3.11' and platform_python_implementation != 'CPython' and sys_platform == 'linux'",
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
        { url = "https://example.com/foo-1.2.3-py3-none-any.whl", marker = "python_version >= '3.11'" },
        { git = "https://example.com/foo/bar", tag = "v1.2.3", marker = "python_version < '3.11'" },
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
        { url = "https://example.com/foo-1.2.3-py3-none-any.whl", marker = "python_version >= '3.11' and platform_python_implementation == 'CPython' and sys_platform == 'darwin'" },
        { index = "supplemental", marker = "python_version < '3.11' and platform_python_implementation != 'CPython' and sys_platform == 'linux'" },
    ]

    [tool.hatch.build.targets.sdist]
    include = [
        "packages-sdist-wheel",
        "packages-sdist-wheel-2",
        "packages-sdist-wheel-3/**/*.py",
        "packages-sdist",
        "packages-sdist-2",
        "from/packages-from",
        "packages-to",
        "from/packages-from-to",
        "packages-glob-to/**/*.py",
        "from/packages-glob-from-to/**/*.py",
        "include-sdist-wheel",
        "include-sdist-wheel-2",
        "include-sdist-wheel-3",
        "include-sdist-wheel-4",
        "include-sdist",
        "include-sdist-2",
    ]
    exclude = [
        "exclude-sdist-wheel",
        "exclude-sdist-wheel-2",
    ]

    [tool.hatch.build.targets.wheel]
    include = [
        "packages-sdist-wheel",
        "packages-sdist-wheel-2",
        "packages-sdist-wheel-3/**/*.py",
        "packages-wheel",
        "packages-wheel-2",
        "from/packages-from",
        "packages-to",
        "from/packages-from-to",
        "packages-glob-to/**/*.py",
        "from/packages-glob-from-to/**/*.py",
        "include-sdist-wheel",
        "include-sdist-wheel-2",
        "include-sdist-wheel-3",
        "include-sdist-wheel-4",
        "include-wheel",
        "include-wheel-2",
    ]
    exclude = [
        "exclude-sdist-wheel",
        "exclude-sdist-wheel-2",
    ]

    [tool.hatch.build.targets.wheel.sources]
    "from/packages-from" = "packages-from"
    packages-to = "to/packages-to"
    "from/packages-from-to" = "to/packages-from-to"
    packages-glob-to = "to/packages-glob-to"
    "from/packages-glob-from-to" = "to/packages-glob-from-to"

    [tool.ruff]
    fix = true

    [tool.ruff.lint]
    # This comment should be preserved.
    fixable = ["I", "UP"]

    [tool.ruff.format]
    preview = true
    "###);

    // Assert that `uv.lock` file was not generated.
    assert!(!project_path.join("uv.lock").exists());
}

#[test]
fn test_dry_run() {
    let project_path = Path::new(FIXTURES_PATH).join("with_lock_file");
    let pyproject = fs::read_to_string(project_path.join("pyproject.toml")).unwrap();

    assert_cmd_snapshot!(cli().arg(&project_path).arg("--dry-run"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Migrated pyproject.toml:
    [project]
    name = "foo"
    version = "0.0.1"
    requires-python = "~=3.11"
    dependencies = ["arrow>=1.2.3,<2"]

    [dependency-groups]
    dev = ["factory-boy>=3.2.1,<4"]
    typing = ["mypy>=1.13.0,<2"]

    [tool.uv]
    package = false
    default-groups = [
        "dev",
        "typing",
    ]
    "###);

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
        .arg("--replace-project-section"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Migrated pyproject.toml:
    [project]
    name = "foo"
    version = "0.0.1"
    description = "A description"
    requires-python = "~=3.11"
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
fn test_pep_621() {
    let project_path = Path::new(FIXTURES_PATH).join("pep_621");

    assert_cmd_snapshot!(cli().arg(&project_path).arg("--dry-run"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Migrated pyproject.toml:
    [build-system]
    requires = ["hatchling"]
    build-backend = "hatchling.build"

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
    "###);
}
