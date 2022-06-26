//! Primitives for manipulating low level details of the OneDrive RET API
use reqwest::{Client, Response, StatusCode};
use serde::Deserialize;
use std::{collections::HashMap, error::Error, fmt::Debug};
type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Deserialize, Debug)]
/// Parsed JSON response data describing the authentication parameters for
/// a OneDrive connection
pub struct Authdata {
    pub token_type: String,
    pub expires_in: u32,
    pub scope: String,
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
}

/// Abstraction around the low level mechanics of the OneDrive REST API
#[derive(Debug, Default)]
pub struct OneDriveApi {
    pub client: Client,
    pub access_token: String,
    pub refresh_token: String,
    pub redirect_uri: String,
    pub client_id: String,
}

impl OneDriveApi {
    pub fn new(
        client: Client,
        client_id: &str,
        redirect_uri: &str,
        access_token: &str,
        refresh_token: &str,
    ) -> Self {
        OneDriveApi {
            client,
            access_token: access_token.to_string(),
            refresh_token: refresh_token.to_string(),
            redirect_uri: redirect_uri.to_string(),
            client_id: client_id.to_string(),
        }
    }

    pub async fn get(&mut self, url: &str) -> MyResult<Response> {
        let result = self
            .client
            .get(url)
            .bearer_auth(&self.access_token)
            .send()
            .await?;
        match result.error_for_status() {
            Ok(resp) => Ok(resp),
            Err(e) => match e.status() {
                Some(StatusCode::UNAUTHORIZED) => {
                    self.refresh_auth_data().await?;
                    let result = self
                        .client
                        .get(url)
                        .bearer_auth(&self.access_token)
                        .send()
                        .await?;
                    match result.error_for_status() {
                        Ok(resp) => Ok(resp),
                        Err(e) => Err(Box::new(e)),
                    }
                }
                _ => Err(Box::new(e)),
            },
        }
    }

    pub async fn refresh_auth_data(&mut self) -> MyResult<()> {
        let client = reqwest::Client::new();

        let url = "https://login.live.com/oauth20_token.srf";

        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert("client_id", &self.client_id);
        params.insert("redirect_uri", &self.redirect_uri);
        params.insert("refresh_token", &self.refresh_token);
        params.insert("grant_type", "refresh_token");
        let response = client.post(url).form(&params).send().await?;
        let data: Authdata = response.json::<Authdata>().await?;
        self.access_token = data.access_token;
        self.refresh_token = data.refresh_token;
        Ok(())
    }
}
