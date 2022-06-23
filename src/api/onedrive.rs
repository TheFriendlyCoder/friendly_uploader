//! Primitives for connecting to the OneDrive service
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
type MyResult<T> = Result<T, Box<dyn Error>>;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DriveItem {
    pub name: String,

    #[serde(flatten)]
    pub extras: HashMap<String, Value>,
}

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

pub struct OneDriveTwo {
    client: Client,
    token: String,
}

impl OneDriveTwo {
    pub fn new(access_token: String) -> Self {
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .gzip(true)
            .build()
            .unwrap();
        OneDriveTwo {
            client,
            token: access_token,
        }
    }

    pub async fn list(&self) -> MyResult<DriveItemList> {
        let url = "https://graph.microsoft.com/v1.0/me/drive/root/children";
        let opt_resp = self.client.get(url).bearer_auth(&self.token).send().await?;

        // TODO: detect non-success error codes and return here
        println!("Status code is {:#?}", opt_resp.status());

        Ok(opt_resp.json().await?)
    }
}
