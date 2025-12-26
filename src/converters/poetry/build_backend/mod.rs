use crate::converters::BuildBackend;
use crate::schema::pyproject::BuildSystem;
use indexmap::IndexMap;
pub mod hatch;
pub mod uv;

type HatchTargetsIncludeAndSource = (
    Option<Vec<String>>,
    Option<Vec<String>>,
    Option<IndexMap<String, String>>,
    Option<IndexMap<String, String>>,
    Option<IndexMap<String, String>>,
);

pub fn get_new_build_system(
    current_build_system: Option<BuildSystem>,
    new_build_system: Option<BuildBackend>,
) -> Option<BuildSystem> {
    if current_build_system?.build_backend? == "poetry.core.masonry.api" {
        return match new_build_system {
            None | Some(BuildBackend::Hatch) => Some(BuildSystem {
                requires: vec!["hatchling".to_string()],
                build_backend: Some("hatchling.build".to_string()),
            }),
            Some(BuildBackend::Uv) => Some(BuildSystem {
                requires: vec!["uv_build".to_string()],
                build_backend: Some("uv_build".to_string()),
            }),
        };
    }
    None
}
