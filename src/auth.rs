pub fn get_url() -> String {
    let redirect_uri = "http://localhost:8080/";
    let client_id = "f9b7e56c-0d02-4ba4-b1ee-24a98f591be4";
    let scope = "onedrive.readwrite offline_access";
    format!("https://login.live.com/oauth20_authorize.srf?client_id={}&scope={}&response_type=code&redirect_uri={}", client_id, scope, redirect_uri)
}
