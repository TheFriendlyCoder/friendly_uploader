//! Primitves for managing OneDrive authentication tokens
//!
//! App registration and configuration can be done through here:
//!     https://aka.ms/AppRegistrations
//!
//! Documentation for the OneDrive authentication process is found here:
//!     https://docs.microsoft.com/en-us/onedrive/developer/rest-api/getting-started/msa-oauth?view=odsp-graph-online
use futures::executor;
use reqwest;
use serde::Deserialize;
use simple_error::SimpleError;
use std::{collections::HashMap, error::Error};
use url::Url;
use urlencoding::encode;

/// URL where the browser will be redirected to after the user approves
/// access to their OneDrive account for our app
const REDIRECT_URI: &str = "http://127.0.0.1:8080/";
/// GUID that uniquely identifies our application to OneDrive
const CLIENT_ID: &str = "f9b7e56c-0d02-4ba4-b1ee-24a98f591be4";

/// Gets a formatted URL that can be pasted into a web browser to request access
/// to the currently logged in OneDrive users profile for our app
pub fn get_auth_url() -> String {
    let scope = encode("onedrive.readwrite offline_access");
    format!("https://login.live.com/oauth20_authorize.srf?client_id={}&scope={}&response_type=code&redirect_uri={}", CLIENT_ID, scope, REDIRECT_URI)
}

/// Parses a short lived authentication token from a URL which is generated
/// from the OneDrive authentication request initiatied by the URL generated
/// by the get_auth_url() helper method
///
/// # Arguments
///
/// * `url` - Response URL produced by the OneDrive authentication process
///           Is expected to have a short lived authentication token encoded
///           in a query parameter named "code"
pub fn parse_token(url: &str) -> Result<Box<str>, Box<dyn Error>> {
    let url_data = Url::parse(url)?;
    for pair in url_data.query_pairs() {
        if pair.0 == "code" {
            return Ok(Box::from(pair.1));
        }
    }
    Err(SimpleError::new("URL did not contain authentication token").into())
}

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

/// Returns authorization data from OneDrive for the current application
///
/// # Arguments
///
/// * `client_code` - temporary authentication code provided by OneDrive after
///                   the user has accepted the authentication request for
///                   the application
pub fn get_auth_data(client_code: &str) -> Result<Authdata, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let url = "https://login.live.com/oauth20_token.srf";
    let mut params = HashMap::new();
    params.insert("client_id", CLIENT_ID);
    params.insert("redirect_uri", REDIRECT_URI);
    params.insert("code", client_code);
    params.insert("grant_type", "authorization_code");

    let response = executor::block_on(client.post(url).form(&params).send())?;
    let data: Authdata = executor::block_on(response.json::<Authdata>())?;
    Ok(data)
}

/// Returns new authentication parameters for OneDrive renewing the auth token
/// expiration in the process
///
/// # Arguments
///
/// * `refresh_token` - temporary authentication token loaded previously which allows
///                     us to request a new, longer term use auth token from OneDrive
///                     Returned auth data will include a new refresh token for use
///                     in subsequent calls
pub fn refresh_auth_data(refresh_token: &str) -> Result<Authdata, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let url = "https://login.live.com/oauth20_token.srf";

    let mut params = HashMap::new();
    params.insert("client_id", CLIENT_ID);
    params.insert("redirect_uri", REDIRECT_URI);
    params.insert("refresh_token", refresh_token);
    params.insert("grant_type", "refresh_token");
    let response = executor::block_on(client.post(url).form(&params).send())?;
    let data: Authdata = executor::block_on(response.json::<Authdata>())?;
    Ok(data)
}
