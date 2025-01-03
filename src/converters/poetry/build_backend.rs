use crate::schema::hatch::{Build, BuildTarget, Hatch};
use crate::schema::poetry::{Format, Include, Package};
use crate::schema::pyproject::BuildSystem;
use crate::schema::utils::SingleOrVec;
use indexmap::IndexMap;
use std::path::PathBuf;

type HatchBuildTargetIncludeSource = (
    Option<Vec<String>>,
    Option<Vec<String>>,
    Option<IndexMap<String, String>>,
    Option<IndexMap<String, String>>,
);

pub fn get_new_build_system(build_system: Option<BuildSystem>) -> Option<BuildSystem> {
    if build_system?.build_backend? == "poetry.core.masonry.api" {
        return Some(BuildSystem {
            requires: vec!["hatchling".to_string()],
            build_backend: Some("hatchling.build".to_string()),
        });
    }
    None
}

pub fn get_hatch(
    packages: Option<&Vec<Package>>,
    include: Option<&Vec<Include>>,
    exclude: Option<&Vec<String>>,
) -> Option<Hatch> {
    let mut targets = IndexMap::new();
    let (sdist_include, wheel_include, sdist_sources, wheel_sources) =
        get_hatch_include(packages, include);

    let sdist_target = BuildTarget {
        include: sdist_include,
        exclude: exclude.cloned(),
        sources: sdist_sources,
    };
    let wheel_target = BuildTarget {
        include: wheel_include,
        exclude: exclude.cloned(),
        sources: wheel_sources,
    };

    if sdist_target != BuildTarget::default() {
        targets.insert("sdist".to_string(), sdist_target);
    }
    if wheel_target != BuildTarget::default() {
        targets.insert("wheel".to_string(), wheel_target);
    }

    if targets.is_empty() {
        return None;
    }

    Some(Hatch {
        build: Some(Build {
            targets: Some(targets),
        }),
    })
}

fn get_hatch_include(
    packages: Option<&Vec<Package>>,
    include: Option<&Vec<Include>>,
) -> HatchBuildTargetIncludeSource {
    let mut sdist_include: Vec<String> = Vec::new();
    let mut sdist_sources: IndexMap<String, String> = IndexMap::new();
    let mut wheel_include: Vec<String> = Vec::new();
    let mut wheel_sources: IndexMap<String, String> = IndexMap::new();

    if let Some(packages) = packages {
        for Package {
            include,
            format,
            from,
            to,
        } in packages
        {
            let mut path = PathBuf::from(from.as_ref().map_or("", |from| from));
            path.push(include);

            let full_include = path.to_string_lossy().to_string();

            match format {
                None => {
                    sdist_include.push(full_include.clone());
                    wheel_include.push(full_include.clone());

                    if let Some(source) =
                        get_hatch_source(include.clone(), to.as_ref(), from.as_ref())
                    {
                        sdist_sources.insert(full_include.clone(), source.clone());
                        wheel_sources.insert(full_include.clone(), source.clone());
                    }
                }
                Some(SingleOrVec::Single(Format::Sdist)) => {
                    sdist_include.push(full_include.clone());

                    if let Some(source) =
                        get_hatch_source(include.clone(), to.as_ref(), from.as_ref())
                    {
                        sdist_sources.insert(full_include.clone(), source);
                    }
                }
                Some(SingleOrVec::Single(Format::Wheel)) => {
                    wheel_include.push(full_include.clone());

                    if let Some(source) =
                        get_hatch_source(include.clone(), to.as_ref(), from.as_ref())
                    {
                        wheel_sources.insert(full_include.clone(), source);
                    }
                }
                Some(SingleOrVec::Vec(vec)) => {
                    if vec.contains(&Format::Sdist) || vec.is_empty() {
                        sdist_include.push(full_include.clone());

                        if let Some(source) =
                            get_hatch_source(include.clone(), to.as_ref(), from.as_ref())
                        {
                            sdist_sources.insert(full_include.clone(), source);
                        }
                    }
                    if vec.contains(&Format::Wheel) || vec.is_empty() {
                        wheel_include.push(full_include.clone());

                        if let Some(source) =
                            get_hatch_source(include.clone(), to.as_ref(), from.as_ref())
                        {
                            wheel_sources.insert(full_include.clone(), source);
                        }
                    }
                }
            }
        }
    }

    if let Some(include) = include {
        for inc in include {
            match inc {
                Include::String(path) | Include::Map { path, format: None } => {
                    sdist_include.push(path.to_string());
                    wheel_include.push(path.to_string());
                }
                Include::Map {
                    path,
                    format: Some(SingleOrVec::Vec(format)),
                } => match format[..] {
                    [] | [Format::Sdist, Format::Wheel] => {
                        sdist_include.push(path.to_string());
                        wheel_include.push(path.to_string());
                    }
                    [Format::Sdist] => sdist_include.push(path.to_string()),
                    [Format::Wheel] => wheel_include.push(path.to_string()),
                    _ => (),
                },
                Include::Map {
                    path,
                    format: Some(SingleOrVec::Single(Format::Sdist)),
                } => sdist_include.push(path.to_string()),
                Include::Map {
                    path,
                    format: Some(SingleOrVec::Single(Format::Wheel)),
                } => wheel_include.push(path.to_string()),
            }
        }
    }

    (
        if sdist_include.is_empty() {
            None
        } else {
            Some(sdist_include)
        },
        if wheel_include.is_empty() {
            None
        } else {
            Some(wheel_include)
        },
        if sdist_sources.is_empty() {
            None
        } else {
            Some(sdist_sources)
        },
        if wheel_sources.is_empty() {
            None
        } else {
            Some(wheel_sources)
        },
    )
}

fn get_hatch_source(include: String, to: Option<&String>, from: Option<&String>) -> Option<String> {
    if let Some(to) = to {
        let mut path = PathBuf::from(to);
        path.push(include);
        return Some(path.to_string_lossy().to_string());
    }

    if from.is_some() {
        return Some(include);
    }

    None
}
