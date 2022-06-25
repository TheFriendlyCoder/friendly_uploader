//! Primitives for connecting to the OneDrive service
//! This submodule iteracts with the REST API to perform its operations. See here for more details:
//!     https://docs.microsoft.com/en-us/onedrive/developer/rest-api/
use reqwest::{Client, Response};
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

#[derive(Debug)]
pub struct OneDrive {
    client: Client,
    token: String,
}

impl Default for OneDrive {
    fn default() -> OneDrive {
        OneDrive::new("".to_string())
    }
}

#[derive(Debug, Deserialize)]
pub struct MetadataItem {
    pub kind: String,
    pub name: String,
    pub url: String,
}
#[derive(Debug, Deserialize)]
pub struct Metadata {
    #[serde(rename = "@odata.context")]
    pub context: String,
    pub value: Vec<MetadataItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "@odata.context")]
    pub context: String,
    pub business_phones: Vec<String>,
    pub display_name: String,
    pub given_name: String,
    pub id: String,
    pub job_title: Option<String>,
    pub mail: Option<String>,
    pub mobile_phone: Option<String>,
    pub offline_location: Option<String>,
    pub preferred_language: Option<String>,
    pub surname: String,
    pub user_principal_name: String,
    #[serde(skip)]
    api: Option<OneDrive>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveQuota {
    pub deleted: usize,
    pub remaining: usize,
    pub state: String,
    pub storage_plan_information: Value,
    pub total: usize,
    pub used: usize,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveMeta {
    #[serde(rename = "@odata.context")]
    pub context: String,
    pub drive_type: String,
    pub id: String,
    // TODO: parse partial content for User object and lazy load remaining data
    pub owner: Value,
    pub quota: DriveQuota,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSystemInfo {
    pub created_date_time: String,
    pub last_modified_date_time: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Drive {
    #[serde(rename = "@odata.context")]
    pub context: String,
    pub c_tag: String,
    pub created_by: Value,
    // TODO: parse into structured date/time format
    pub created_date_time: String,
    pub e_tag: String,
    pub file_system_info: FileSystemInfo,
    pub folder: Value,
    pub id: String,
    pub last_modified_by: Value,
    pub last_modified_date_time: String,
    pub name: String,
    // TODO: Parse into parent drive/folder object
    pub parent_reference: Value,
    pub root: Value,
    pub size: usize,
    pub web_url: String,
    #[serde(skip)]
    api: Option<OneDrive>,
}

impl User {
    pub async fn new(onedrive: OneDrive, resp: Response) -> User {
        let mut retval: User = resp.json().await.unwrap();
        retval.api = Some(onedrive);
        retval
    }
    fn api(&self) -> &OneDrive {
        self.api.as_ref().unwrap()
    }
    pub async fn drive(&self) -> MyResult<DriveMeta> {
        // TODO: extract entity URL to a private struct property
        // TODO: figure out how to lazy load properties in a struct
        let url = "https://graph.microsoft.com/v1.0/me/drive";
        let opt_resp = self
            .api()
            .client
            .get(url)
            .bearer_auth(&self.api().token)
            .send()
            .await?
            .error_for_status()?;
        Ok(opt_resp.json().await?)
    }

    pub async fn root(&self) -> MyResult<Drive> {
        // TODO: extract entity URL to a private struct property
        // TODO: figure out how to lazy load properties in a struct
        let url = "https://graph.microsoft.com/v1.0/me/drive/root";
        let opt_resp = self
            .api()
            .client
            .get(url)
            .bearer_auth(&self.api().token)
            .send()
            .await?
            .error_for_status()?;
        let temp = OneDrive::new(self.api().token.clone());
        Ok(Drive::new(temp, opt_resp).await)
    }
}

impl Drive {
    pub async fn new(onedrive: OneDrive, resp: Response) -> Drive {
        let mut retval: Drive = resp.json().await.unwrap();
        retval.api = Some(onedrive);
        retval
    }
    fn api(&self) -> &OneDrive {
        self.api.as_ref().unwrap()
    }
    pub async fn children(&self) -> MyResult<DriveItemList> {
        let url = "https://graph.microsoft.com/v1.0/me/drive/root/children";
        // TODO: move all mechanics for REST API into a new api / protocols struct
        let opt_resp = self
            .api()
            .client
            .get(url)
            .bearer_auth(&self.api().token)
            .send()
            .await?
            .error_for_status()?;
        println!("{:#?}", opt_resp.headers());
        let retval = opt_resp.json().await?;
        //let retval = opt_resp.text().await?;
        println!("{:#?}", retval);
        Ok(retval)
    }
}
impl OneDrive {
    pub fn new(access_token: String) -> Self {
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .gzip(true)
            .build()
            .unwrap();
        OneDrive {
            client,
            token: access_token,
        }
    }

    pub async fn meta(&self) -> MyResult<Metadata> {
        // NOTE: meta data is loadable without authentication
        //       could be used for connection verification
        let url = "https://graph.microsoft.com/v1.0";
        let opt_resp = self
            .client
            .get(url)
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;
        Ok(opt_resp.json().await?)
    }

    pub async fn me(&self) -> MyResult<User> {
        // Requires user.read scope
        let url = "https://graph.microsoft.com/v1.0/me";
        let opt_resp = self
            .client
            .get(url)
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;
        // TODO: figure out how to get a shared reference to the OneDrive API
        //       across structs
        let temp = OneDrive::new(self.token.clone());
        Ok(User::new(temp, opt_resp).await)
    }

    //pub async fn list(&self) -> MyResult<DriveItemList> {
    //pub async fn list(&self) -> MyResult<serde_json::Value> {
    //pub async fn list(&self) -> MyResult<String> {
    pub async fn list(&self) -> MyResult<Drive> {
        //let url = "https://graph.microsoft.com/v1.0/me/drive/root/children";
        let url = "https://graph.microsoft.com/v1.0/me/drive/root";
        let opt_resp = self
            .client
            .get(url)
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;
        println!("{:#?}", opt_resp.headers());
        let retval = opt_resp.json().await?;
        //let retval = opt_resp.text().await?;
        println!("{:#?}", retval);
        Ok(retval)
    }
}
