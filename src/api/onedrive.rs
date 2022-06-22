//! Primitives for connecting to the OneDrive service
use reqwest::{Client, Response};
use std::error::Error;
type MyResult<T> = Result<T, Box<dyn Error>>;

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

    pub async fn list(&self) -> MyResult<reqwest::Response> {
        let url = "https://graph.microsoft.com/v1.0/me/drive/root/children";
        let opt_resp = self
            .client
            //.get(api_url![&self.drive, &item.into(), "children"])
            .get(url)
            //.apply(option)
            .bearer_auth(&self.token)
            .send()
            .await?;
        //.parse_optional()
        //.await?;
        Ok(opt_resp)
    }
}
