//! Primitives for manipulating OneDrive user entities
use reqwest::Response;
use serde::Deserialize;

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
}

impl User {
    /// Constructs a new instance of a User struct
    ///
    /// # Arguments
    ///
    /// * `resp` - HTTP response describing the OneDrive user
    pub async fn new(resp: Response, url: &str) -> User {
        let mut retval: User = resp.json().await.unwrap();
        retval.url = url.to_string();
        retval
    }
}
