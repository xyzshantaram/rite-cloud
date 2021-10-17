use std::string;

use oauth2::basic::BasicClient;
use oauth2::curl::http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, RequestTokenError,
    Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct UserInfoResponse {
    // email: String,
    id: String,
    given_name: String,
}

#[derive(Debug, Deserialize)]
struct AuthRequestQuery {
    code: String,
    state: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct GhResponse {
    login: String,
}

use super::OauthConfig;
use crate::rite::server_error;
use crate::State;
use tide::{Redirect, Request};

pub async fn gh(req: Request<State>) -> tide::Result {
    let client = &req.state().gh_client;
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    Ok(Redirect::see_other(auth_url).into())
}

pub async fn gh_authorized(mut req: Request<State>) -> tide::Result {
    let state = &req.state();
    let client = &state.gh_client;
    let tera = state.tera.clone();
    let query: AuthRequestQuery = req.query()?;
    let code = AuthorizationCode::new(query.code);
    let token_res = client.exchange_code(code).request(http_client);
    match token_res {
        Ok(token) => {
            let token_str = token.access_token().secret();
            let res: GhResponse = surf::get("https://api.github.com/user")
                .header("Authorization", format!("token {}", token_str))
                .recv_json::<GhResponse>()
                .await?;
            let session = req.session_mut();
            match session.insert("username", &res.login) {
                Ok(_) => {
                    tide::log::info!("User authorised, redirecting...");
                    Ok(Redirect::new("/").into())
                }
                Err(e) => {
                    server_error(tera, "Could not log in", &format!("error saving session: {:?}", e))
                }
            }
        }
        Err(RequestTokenError::Parse(_, bytes)) => {
            return server_error(
                tera,
                "Expired or invalid code",
                &format!(
                    "error text: {}",
                    string::String::from_utf8(bytes).unwrap_or_default()
                ),
            );
        }
        Err(otherwise) => {
            return server_error(
                tera,
                "Error while getting access token",
                &format!("{:?}", otherwise),
            );
        }
    }
}

pub async fn logout(mut req: Request<State>) -> tide::Result {
    let session = req.session_mut();
    session.destroy();
    Ok(Redirect::new("/").into())
}

pub fn gh_oauth_client(cfg: &OauthConfig) -> tide::Result<BasicClient> {
    Ok(BasicClient::new(
        ClientId::new(cfg.client_id.clone()),
        Some(ClientSecret::new(cfg.client_secret.clone())),
        AuthUrl::new(cfg.auth_url.clone())?,
        Some(TokenUrl::new(cfg.token_url.clone())?),
    )
    .set_redirect_url(RedirectUrl::new(cfg.redirect_url.clone())?))
}
