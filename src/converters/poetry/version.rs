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
                [x, _, ..] | [x] => Self::Inclusive(version, Version::new([x + 1])),
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

        // Poetry accepts space-separated version clauses (e.g., ">=3.10 <4.0"), but PEP 440
        // requires comma-separated clauses. Try normalizing spaces to commas.
        // See: https://python-poetry.org/docs/dependency-specification/#version-constraints
        let normalized = s.replace(' ', ",");
        if normalized != s && VersionSpecifiers::from_str(&normalized).is_ok() {
            return Ok(Self::String(normalized));
        }

        let mut pep_440_specifier = Vec::new();

        // Even when using Poetry-specific version specifiers, it is still possible to define
        // additional PEP 440 specifiers (e.g., "^1.0,!=1.1.0") or even define multiple Poetry
        // specifiers (e.g., "^1.0,^1.1"), so we need to split over "," and treat each group
        // separately, knowing that each group can either be a Poetry-specific specifier, or a PEP
        // 440 one.
        for specifier in s.split(',') {
            let specifier = specifier.trim();

            // If the subgroup is a valid PEP 440 specifier, we can directly use it without any
            // transformation.
            if VersionSpecifiers::from_str(specifier).is_ok() {
                pep_440_specifier.push(Self::String(specifier.to_string()));
            } else {
                match specifier.split_at(1) {
                    ("*", "") => pep_440_specifier.push(Self::String(String::new())),
                    ("^", version) => pep_440_specifier.push(Self::from_caret(version.trim())),
                    ("~", version) => pep_440_specifier.push(Self::from_tilde(version.trim())),
                    ("=", version) => {
                        pep_440_specifier.push(Self::String(format!("=={version}")));
                    }
                    _ => pep_440_specifier.push(Self::String(format!("=={s}"))),
                }
            }
        }

        Ok(Self::String(
            pep_440_specifier
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(","),
        ))
    }
}

impl std::fmt::Display for PoetryPep440 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match &self {
            Self::String(s) => s.clone(),
            Self::Compatible(version) => format!("~={version}"),
            Self::Matching(version) => format!("=={version}"),
            Self::Inclusive(lower, upper) => format!(">={lower},<{upper}"),
        };

        write!(f, "{str}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_separated_version_constraints() {
        // Poetry accepts space-separated constraints, should convert to comma-separated
        let result = PoetryPep440::from_str(">=3.10 <4.0").unwrap();
        assert_eq!(result.to_string(), ">=3.10,<4.0");
    }

    #[test]
    fn test_space_separated_three_constraints() {
        let result = PoetryPep440::from_str(">=3.10 <4.0 !=3.11").unwrap();
        assert_eq!(result.to_string(), ">=3.10,<4.0,!=3.11");
    }

    #[test]
    fn test_comma_separated_preserved() {
        // Already comma-separated should be preserved
        let result = PoetryPep440::from_str(">=3.10,<4.0").unwrap();
        assert_eq!(result.to_string(), ">=3.10,<4.0");
    }

    #[test]
    fn test_single_constraint() {
        let result = PoetryPep440::from_str(">=3.10").unwrap();
        assert_eq!(result.to_string(), ">=3.10");
    }

    #[test]
    fn test_caret_operator() {
        let result = PoetryPep440::from_str("^3.10").unwrap();
        assert_eq!(result.to_string(), ">=3.10,<4");
    }
}
