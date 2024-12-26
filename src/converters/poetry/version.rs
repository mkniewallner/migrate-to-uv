use pep440_rs::{Version, VersionSpecifiers};
use std::str::FromStr;

pub enum PoetryPep440 {
    String(String),
    Compatible(Version),
    Matching(Version),
    Inclusive(Version, Version),
}

impl PoetryPep440 {
    pub fn to_python_marker(&self) -> String {
        let pep_440_python = VersionSpecifiers::from_str(self.to_string().as_str()).unwrap();

        pep_440_python
            .iter()
            .map(|spec| format!("python_version {} '{}'", spec.operator(), spec.version()))
            .collect::<Vec<String>>()
            .join(" and ")
    }

    /// <https://python-poetry.org/docs/dependency-specification/#caret-requirements>
    fn from_caret(s: &str) -> Self {
        if let Ok(version) = Version::from_str(s) {
            return match version.clone().release() {
                [0, 0, z] => Self::Inclusive(version, Version::new([0, 0, z + 1])),
                [0, y] | [0, y, _, ..] => Self::Inclusive(version, Version::new([0, y + 1])),
                [x, _, _, ..] | [x] => Self::Inclusive(version, Version::new([x + 1])),
                [_, _] => Self::Compatible(version),
                [..] => Self::String(String::new()),
            };
        }
        Self::Matching(Version::from_str(s).unwrap())
    }

    /// <https://python-poetry.org/docs/dependency-specification/#tilde-requirements>
    fn from_tilde(s: &str) -> Self {
        if let Ok(version) = Version::from_str(s) {
            return match version.clone().release() {
                [_, _, _, ..] => Self::Compatible(version),
                [x, y] => Self::Inclusive(version, Version::new([x, &(y + 1)])),
                [x] => Self::Inclusive(version, Version::new([x + 1])),
                [..] => Self::String(String::new()),
            };
        }
        Self::Matching(Version::from_str(s).unwrap())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParsePep440Error;

impl FromStr for PoetryPep440 {
    type Err = ParsePep440Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // While Poetry has its own specification for version specifiers, it also supports most of
        // the version specifiers defined by PEP 440. So if the version is a valid PEP 440
        // definition, we can directly use it without any transformation.
        if VersionSpecifiers::from_str(s).is_ok() {
            return Ok(Self::String(s.to_string()));
        }

        match s.split_at(1) {
            ("*", "") => Ok(Self::String(String::new())),
            ("^", version) => Ok(Self::from_caret(version.trim())),
            ("~", version) => Ok(Self::from_tilde(version.trim())),
            _ => Ok(Self::String(format!("=={s}"))),
        }
    }
}

impl std::fmt::Display for PoetryPep440 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match &self {
            Self::String(s) => s.to_string(),
            Self::Compatible(version) => format!("~={version}"),
            Self::Matching(version) => format!("=={version}"),
            Self::Inclusive(lower, upper) => format!(">={lower},<{upper}"),
        };

        write!(f, "{str}")
    }
}
