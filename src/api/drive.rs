//! Primitives for manipulating OneDrive drives
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use reqwest::Response;
use serde::Deserialize;
use serde_json::Value;

use super::driveitem::DriveItemList;
use super::onedriveapi::OneDriveApi;
type MyResult<T> = Result<T, Box<dyn Error>>;

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
    url: String,
    #[serde(skip)]
    api: Option<Rc<RefCell<OneDriveApi>>>,
}

impl Drive {
    /// Constructs new instances of the Drive struct
    ///
    /// # Arguments
    ///
    /// * `resp` - HTTP response data loaded from the OneDrive API
    /// * `url` - Full URL to the REST API endpoint managed by this entity
    /// * `api` - Shared reference to the interface used to communicate with
    ///           the OneDrive REST API
    pub async fn new(resp: Response, url: &str, api: Rc<RefCell<OneDriveApi>>) -> Drive {
        let mut retval: Drive = resp.json().await.unwrap();
        retval.url = url.to_string();
        retval.api = Some(api);
        retval
    }

    /// Helper method that unwraps a reference to the REST API interface used
    /// by the impl for making dynamic API calls
    fn api(&self) -> &Rc<RefCell<OneDriveApi>> {
        self.api.as_ref().unwrap()
    }

    /// Gets a list of 0 or more drive items contained within this drive
    pub async fn children(&self) -> MyResult<DriveItemList> {
        let url = format!("{}{}", self.url, "/children");
        // TODO: move all mechanics for REST API into a OneDriveApi
        let opt_resp = self
            .api()
            .borrow()
            .client
            .get(url)
            .bearer_auth(&self.api().borrow().access_token)
            .send()
            .await?
            .error_for_status()?;
        Ok(opt_resp.json().await?)
    }
}
