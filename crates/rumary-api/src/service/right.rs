use crate::error::{AppError, AppResult};
use rumary_dto::domain::api::{RightFromRow, RightId, RightKey};
use std::collections::HashMap;

pub struct Rights {
    rights_ids: Vec<RightId>,
    rights_keys: Vec<RightKey>,
    default_values: Vec<bool>,
    index_by_key: HashMap<RightKey, usize>,
}

impl Rights {
    pub fn get_right(&self, key: &RightKey) -> AppResult<RightId> {
        let index = self.get_index(key)?;

        Ok(self.rights_ids[index])
    }

    fn get_index(&self, right_key: &RightKey) -> AppResult<usize> {
        self.index_by_key
            .get(right_key)
            .copied()
            .ok_or(AppError::NotFound(
                "Cannot get index with RightKey does not exist".to_string(),
            ))
    }

    pub fn rights_ids(&self) -> Vec<RightId> {
        self.rights_ids.clone()
    }

    pub fn rights_keys(&self) -> Vec<RightKey> {
        self.rights_keys.clone()
    }
    pub fn default_values(&self) -> Vec<bool> {
        self.default_values.clone()
    }
}

impl From<RightFromRow> for Rights {
    fn from(value: RightFromRow) -> Self {
        let mut entries = value
            .0
            .into_iter()
            .filter(|(_, definition)| definition.active)
            .map(|(key, definition)| (RightKey::new(key), definition))
            .collect::<Vec<_>>();
        entries.sort_by_key(|(_, definition)| definition.id);

        let mut rights_ids = Vec::with_capacity(entries.len());
        let mut rights_keys = Vec::with_capacity(entries.len());
        let mut default_values = Vec::with_capacity(entries.len());
        let mut index_by_key = HashMap::with_capacity(entries.len());

        for (index, (right_key, definition)) in entries.into_iter().enumerate() {
            rights_ids.push(definition.id);
            index_by_key.insert(right_key.clone(), index);
            rights_keys.push(right_key);
            default_values.push(definition.default);
        }

        Self {
            rights_ids,
            rights_keys,
            default_values,
            index_by_key,
        }
    }
}
