//! Primary entry point for the module
//! Defines the basic connection and authentication interface for OneDrive

use crate::api::user::User;
use reqwest::Client;
use std::{error::Error, fmt::Debug};
type MyResult<T> = Result<T, Box<dyn Error>>;

const ONEDRIVE_API_URL: &str = "https://graph.microsoft.com/v1.0";

#[derive(Debug)]
/// Primary entry point for configuring interactions with OneDrive
/// All subsequent OneDrive operations are expected to be initiated
/// through this struct
pub struct OneDrive {
    client: Client,
    access_token: String,
}

impl OneDrive {
    /// Constructs a new instance of our OneDrive API client
    ///
    /// # Arguments
    ///
    /// * `token` - API key used to authenticate with.
    pub fn new(token: &str) -> Self {
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .gzip(true)
            .build()
            .unwrap();
        OneDrive {
            client,
            access_token: token.to_string(),
        }
    }

    /// Retrieves data for the currently logged in user
    pub async fn me(&self) -> MyResult<User> {
        // Requires user.read scope
        let url = format!("{}/{}", ONEDRIVE_API_URL, "me");
        let opt_resp = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await?
            .error_for_status()?;
        Ok(User::new(opt_resp, &url).await)
    }
}
