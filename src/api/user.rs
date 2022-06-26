//! Primitives for manipulating OneDrive user entities
use std::error::Error;
use std::rc::Rc;

use reqwest::Response;
use serde::Deserialize;

use super::drive::Drive;
use super::onedrive::OneDriveApi;
type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Abstraction around a OneDrive user
/// See API docs for more details
///     https://docs.microsoft.com/en-ca/graph/api/resources/user
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
    url: String,
    #[serde(skip)]
    api: Option<Rc<OneDriveApi>>,
}

impl User {
    /// Constructs new instances of the User struct
    ///
    /// # Arguments
    ///
    /// * `resp` - HTTP response data loaded from the OneDrive API
    /// * `url` - Full URL to the REST API endpoint managed by this entity
    /// * `api` - Shared reference to the interface used to communicate with
    ///           the OneDrive REST API
    pub async fn new(resp: Response, url: &str, api: Rc<OneDriveApi>) -> User {
        let mut retval: User = resp.json().await.unwrap();
        retval.url = url.to_string();
        retval.api = Some(api);
        retval
    }

    /// Helper method that unwraps a reference to the REST API interface used
    /// by the impl for making dynamic API calls
    fn api(&self) -> &Rc<OneDriveApi> {
        self.api.as_ref().unwrap()
    }

    /// Gets a reference to the root drive associaed with this user
    /// This interface provides the tools needed to interact with
    /// files and folders contained within the drive
    pub async fn root(&self) -> MyResult<Drive> {
        // TODO: figure out how to lazy load properties in a struct
        let url = format!("{}{}", self.url, "/drive/root");
        let opt_resp = self
            .api()
            .client
            .get(&url)
            .bearer_auth(&self.api().access_token)
            .send()
            .await?
            .error_for_status()?;
        Ok(Drive::new(opt_resp, &url, Rc::clone(self.api())).await)
    }
}
