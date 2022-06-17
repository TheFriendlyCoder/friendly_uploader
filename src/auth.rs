use simple_error::SimpleError;
use std::error::Error;
use url::Url;
use urlencoding::encode;

/// Gets a formatted URL that can be pasted into a web browser to request access
/// to the currently logged in OneDrive users profile for our app
pub fn get_auth_url() -> String {
    let redirect_uri = "http://localhost:8080/";
    let client_id = "f9b7e56c-0d02-4ba4-b1ee-24a98f591be4";
    let scope = encode("onedrive.readwrite offline_access");
    format!("https://login.live.com/oauth20_authorize.srf?client_id={}&scope={}&response_type=code&redirect_uri={}", client_id, scope, redirect_uri)
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
