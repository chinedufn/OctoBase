use super::*;

use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use utoipa::ToSchema;
use yrs::{
    block::{Item, ItemContent, ID},
    types::TypePtr,
    Doc, StateVector,
};

struct ParentMap(HashMap<ID, String>);

impl ParentMap {
    fn from(items: &Vec<&Item>) -> Self {
        let mut name_map: HashMap<ID, String> = HashMap::new();
        let mut padding_ptr: VecDeque<(&Item, usize)> =
            VecDeque::from(items.iter().map(|i| (i.clone(), 0)).collect::<Vec<_>>());

        while let Some((item, retry)) = padding_ptr.pop_back() {
            if retry > 5 {
                debug!("retry failed: {:?}, {:?}, {:?}", item, retry, padding_ptr);
                break;
            }
            let parent = match &item.parent {
                TypePtr::Unknown => "unknown".to_owned(),
                TypePtr::Branch(ptr) => {
                    if let Some(name) = ptr.item_id().and_then(|item_id| name_map.get(&item_id)) {
                        name.clone()
                    } else {
                        padding_ptr.push_front((item, retry + 1));
                        continue;
                    }
                }
                TypePtr::Named(name) => name.to_string(),
                TypePtr::ID(ptr_id) => {
                    if let Some(name) = name_map.get(&ptr_id) {
                        name.clone()
                    } else {
                        padding_ptr.push_front((item, retry + 1));
                        continue;
                    }
                }
            };

            let parent = if let Some(parent_sub) = &item.parent_sub {
                format!("{}.{}", parent, parent_sub)
            } else {
                parent
            };

            name_map.insert(item.id, parent.clone());
        }

        Self(name_map)
    }

    fn get(&self, id: &ID) -> Option<String> {
        self.0.get(id).cloned()
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct History {
    id: String,
    parent: String,
    content: String,
}

pub fn parse_history_client(doc: &Doc) -> Option<String> {
    let update = doc.encode_state_as_update_v1(&StateVector::default());
    if let Ok(update) = Update::decode_v1(&update) {
        serde_json::to_string(
            &update
                .as_items()
                .iter()
                .map(|i| i.id.client)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .ok()
    } else {
        None
    }
}

pub fn parse_history(doc: &Doc, client: u64) -> Option<String> {
    let update = doc.encode_state_as_update_v1(&StateVector::default());
    let update = Update::decode_v1(&update).ok()?;
    let items = update.as_items();

    let mut histories = vec![];
    let parent_map = ParentMap::from(&items);

    for item in items {
        if let ItemContent::Deleted(_) = item.content {
            continue;
        }
        if let Some(parent) = parent_map.get(&item.id) {
            if item.id.client == client || client == 0 {
                let id = format!("{}:{}", item.id.clock, item.id.client);
                histories.push(History {
                    id,
                    parent,
                    content: item.content.to_string(),
                })
            }
        } else {
            info!("headless id: {:?}", item.id);
        }
    }

    serde_json::to_string(&histories).ok()
}
