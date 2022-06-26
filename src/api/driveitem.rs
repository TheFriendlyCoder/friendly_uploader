//! Primitives for manipulating OneDrive drive items
use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

/// Abstraction around a single drive item entity from OneDrive
#[derive(Debug, Clone, Deserialize)]
pub struct DriveItem {
    pub name: String,

    #[serde(flatten)]
    pub extras: HashMap<String, Value>,
}

/// List of OneDrive drive item entities providing a lazy-loading iterator interface
/// to simplify interactions with the API
#[derive(Debug, Deserialize)]
pub struct DriveItemList {
    #[serde(rename = "value")]
    pub data: Vec<DriveItem>,
    #[serde(rename = "@odata.nextLink")]
    pub next_url: Option<String>,
    #[serde(rename = "@odata.deltaLink")]
    pub delta_url: Option<String>,
    #[serde(skip)]
    cur_index: usize,
}

impl Iterator for DriveItemList {
    type Item = DriveItem;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_index == self.data.len() {
            return None;
        }
        // TODO: figure out if we should return a borrowed reference or what
        // TODO: replace implementation with a shared iter of the 'data' vec
        let retval = self.data[self.cur_index].clone();
        self.cur_index += 1;
        Some(retval)
    }
}
