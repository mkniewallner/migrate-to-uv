use indexmap::IndexMap;

pub fn non_empty_vec<T>(vec: Vec<T>) -> Option<Vec<T>> {
    if vec.is_empty() {
        return None;
    }
    Some(vec)
}

pub fn non_empty_index_map<T, U>(map: IndexMap<T, U>) -> Option<IndexMap<T, U>> {
    if map.is_empty() {
        return None;
    }
    Some(map)
}
