//! Primitves for managing OneDrive authentication tokens
//! Used primarily to orchestrate the OAuth authentication
//! process using the "code flow" defined here:
//!     https://docs.microsoft.com/en-us/onedrive/developer/rest-api/getting-started/msa-oauth?view=odsp-graph-online
use futures::executor;
use reqwest;
use serde::Deserialize;
use simple_error::SimpleError;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::{collections::HashMap, error::Error};
use url::Url;
use urlencoding::encode;

/// URL where the browser will be redirected to after the user approves
/// access to their OneDrive account for our app
pub const REDIRECT_URI: &str = "http://127.0.0.1:8080/";
/// GUID that uniquely identifies our application to OneDrive
/// Managed through the Azue app port here:
///     https://portal.azure.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade
const CLIENT_ID: &str = "454dddcf-522d-43b6-b078-b38657e8045a";

/// Gets a formatted URL that can be pasted into a web browser to request access
/// to the currently logged in OneDrive users profile for our app
pub fn get_auth_url() -> String {
    let scope = encode("files.readwrite.all onedrive.readwrite offline_access");
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

/// Retrieves an oauth token for the OneDrive service for the user by opening
/// the OAuth registration page in the default web browser and listening for
/// an approved response from the default listening port on the local machine
pub fn get_oauth_token_from_browser() -> Result<Box<str>, Box<dyn Error>> {
    // TODO: Write tests for this code
    // Reference implementation:
    // https://github.com/ramosbugs/oauth2-rs/blob/main/examples/msgraph.rs
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    open::that(get_auth_url())?;

    let mut params = String::new();
    if let Some(stream) = listener.incoming().next() {
        let mut stream = stream?;

        let mut reader = BufReader::new(&stream);

        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;

        let temp_err = SimpleError::new(format!("Unvalid input line {}", request_line));
        let redirect_url = request_line.split_whitespace().nth(1).ok_or(temp_err)?;
        params.push_str(redirect_url);

        // TODO: update this response to pop up a modal dialog in the browser informing
        //       the user to go back to the terminal, and then force-close the browser tab
        let content = "Go back to your terminal :)";
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
            content.len(),
            content
        );
        stream.write_all(response.as_bytes())?;
    }
    let retval = format!("{}{}", REDIRECT_URI, params);
    Ok(Box::from(retval))
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
