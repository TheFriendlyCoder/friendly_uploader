//! Primary entry point for the module
//! Defines the basic connection and authentication interface for OneDrive

use super::onedriveapi::OneDriveApi;
use super::user::User;

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::Value;
use simple_error::SimpleError;
use std::{cell::RefCell, collections::HashMap, error::Error, fmt::Debug, rc::Rc};
type MyResult<T> = Result<T, Box<dyn Error>>;

const ONEDRIVE_API_URL: &str = "https://graph.microsoft.com/v1.0";

#[derive(Debug)]
/// Primary entry point for configuring interactions with OneDrive
/// All subsequent OneDrive operations are expected to be initiated
/// through this struct
pub struct OneDrive {
    api: Rc<RefCell<OneDriveApi>>,
}

#[derive(Debug, Deserialize)]
pub struct InnerError {
    pub date: String,
    #[serde(rename = "request-id")]
    pub request_id: String,
    #[serde(rename = "client-request-id")]
    pub client_request_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDetail {
    pub message: String,
    pub code: String,
    pub inner_error: InnerError,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

impl OneDrive {
    /// Constructs a new instance of our OneDrive API client
    ///
    /// # Arguments
    ///
    /// * `token` - API key used to authenticate with.
    pub fn new(
        client_id: &str,
        redirect_uri: &str,
        access_token: &str,
        refresh_token: &str,
    ) -> Self {
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .gzip(true)
            .build()
            .unwrap();
        OneDrive {
            api: Rc::new(RefCell::new(OneDriveApi::new(
                client,
                client_id,
                redirect_uri,
                access_token,
                refresh_token,
            ))),
        }
    }

    /// Retrieves profile data for the currently logged in user
    pub async fn me(&self) -> MyResult<User> {
        // Requires user.read scope
        let url = format!("{}{}", ONEDRIVE_API_URL, "/me");
        let opt_resp = self.api.borrow_mut().get(&url).await?;
        // let d2 = match opt_resp.status() {
        //     StatusCode::FORBIDDEN => {}
        //     StatusCode::OK => opt_resp,
        // };
        // let data = match opt_resp.error_for_status() {
        //     Ok(resp) => resp,
        //     Err(e) => {
        //         println!("{:#?}", e);
        //         self.api
        //             .borrow()
        //             .client
        //             .get(&url)
        //             .bearer_auth(&self.api.borrow().access_token)
        //             .send()
        //             .await?
        //     }
        // };
        //let headers = opt_resp.headers().clone();
        //let text: ErrorResponse = opt_resp.json().await?;

        // println!("{:#?}", data);
        // if data.error.code == "InvalidAuthenticationToken" {
        //     self.api.borrow_mut().refresh_auth_data().await?;
        //     let opt_resp = self
        //         .api
        //         .borrow()
        //         .client
        //         .get(&url)
        //         .bearer_auth(&self.api.borrow().access_token)
        //         .send()
        //         .await?;
        // }
        // let temp = match opt_resp.error_for_status() {
        //     Ok(resp) => resp,
        //     Err(e) => {
        //         println!("{:#?}", e);
        //         println!("{:#?}", headers);
        //         return Err(Box::new(e));
        //     }
        // };
        //Err(SimpleError::new("URL did not contain authentication token").into())
        Ok(User::new(opt_resp, &url, Rc::clone(&self.api)).await)
    }
}
