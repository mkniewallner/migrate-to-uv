use toml_edit::visit_mut::{
    VisitMut, visit_array_mut, visit_item_mut, visit_table_like_kv_mut, visit_table_mut,
};
use toml_edit::{Array, InlineTable, Item, KeyMut, Value};

#[derive(Default)]
pub struct PyprojectPrettyFormatter {
    pub parent_keys: Vec<String>,
}

/// Prettifies Pyproject TOML based on usual conventions in the ecosystem.
impl VisitMut for PyprojectPrettyFormatter {
    fn visit_item_mut(&mut self, node: &mut Item) {
        let parent_keys: Vec<&str> = self.parent_keys.iter().map(AsRef::as_ref).collect();

        // Uv indexes are usually represented as array of tables (https://docs.astral.sh/uv/configuration/indexes/).
        if let ["tool", "uv", "index"] = parent_keys.as_slice() {
            let new_node = std::mem::take(node);
            let new_node = new_node
                .into_array_of_tables()
                .map_or_else(|i| i, Item::ArrayOfTables);

            *node = new_node;
        }

        visit_item_mut(self, node);
    }

    fn visit_table_mut(&mut self, node: &mut toml_edit::Table) {
        node.decor_mut().clear();

        if !node.is_empty() {
            node.set_implicit(true);
        }

        visit_table_mut(self, node);
    }

    fn visit_table_like_kv_mut(&mut self, mut key: KeyMut<'_>, node: &mut Item) {
        self.parent_keys.push(key.to_string());

        // Convert some inline tables into tables, when those tables are usually represented as
        // plain tables in the ecosystem.
        if let Item::Value(Value::InlineTable(inline_table)) = node {
            let parent_keys: Vec<&str> = self.parent_keys.iter().map(AsRef::as_ref).collect();

            if matches!(
                parent_keys.as_slice(),
                ["build-system" | "project" | "dependency-groups"]
                    | [
                        "project",
                        "urls"
                            | "optional-dependencies"
                            | "scripts"
                            | "gui-scripts"
                            | "entry-points"
                    ]
                    | ["project", "entry-points", _]
                    | ["tool", "uv"]
                    | ["tool", "uv", "sources"]
                    | ["tool", "hatch", ..]
            ) {
                let position = match parent_keys.as_slice() {
                    ["project"] => Some(0),
                    ["dependency-groups"] => Some(1),
                    ["tool", "uv"] => Some(2),
                    ["tool", "hatch"] => Some(3),
                    _ => None,
                };

                let inline_table = std::mem::replace(inline_table, InlineTable::new());
                let mut table = inline_table.into_table();

                if let Some(position) = position {
                    table.set_position(position);
                }

                key.fmt();
                *node = Item::Table(table);
            }
        }

        visit_table_like_kv_mut(self, key, node);

        self.parent_keys.pop();
    }

    fn visit_array_mut(&mut self, node: &mut Array) {
        visit_array_mut(self, node);

        let parent_keys: Vec<&str> = self.parent_keys.iter().map(AsRef::as_ref).collect();

        // It is common to have each array item on its own line if the array contains more than 2
        // items, so this applies this format on sections that were added. Targeting specific
        // sections ensures that unrelated sections are left intact.
        if matches!(
            parent_keys.as_slice(),
            ["project" | "dependency-groups", ..] | ["tool", "uv" | "hatch", ..]
        ) && node.len() >= 2
        {
            for item in node.iter_mut() {
                item.decor_mut().set_prefix("\n    ");
            }

            node.set_trailing_comma(true);
            node.set_trailing("\n");
        }
    }
}
