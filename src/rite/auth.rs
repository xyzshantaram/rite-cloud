use std::string;

use http_types::StatusCode;
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, RequestTokenError,
    Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct AuthRequestQuery {
    code: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct GhResponse {
    login: String,
}

use super::RiteConfig;
use crate::rite::render_error;
use crate::{State, TERA};
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
    println!("here");
    let state = &req.state();
    let client = &state.gh_client;
    let tera = TERA.clone();
    println!("here2");
    let query: AuthRequestQuery = req.query()?;
    let code = AuthorizationCode::new(query.code);
    let token_res = client.exchange_code(code).request(http_client);
    println!("here3 token_res {:#?}", token_res);
    match token_res {
        Ok(token) => {
            println!("here4");
            let token_str = token.access_token().secret();
            let res_text = reqwest::Client::new()
                .get("https://api.github.com/user")
                .header("Authorization", format!("token {}", token_str))
                .header("User-Agent", "rite-cloud")
                .send()
                .await?
                .text()
                .await?;
            println!("{}", res_text);
            let res: GhResponse = serde_json::from_str(&res_text)?;

            let session = req.session_mut();
            match session.insert("username", &res.login) {
                Ok(_) => {
                    tide::log::info!("User authorised, redirecting...");
                    Ok(Redirect::new("/home").into())
                }
                Err(e) => render_error(
                    &tera,
                    "Could not log in",
                    &format!("error saving session: {:?}", e),
                    StatusCode::InternalServerError,
                ),
            }
        }
        Err(RequestTokenError::Parse(v, bytes)) => {
            println!("here4 {}", v);
            render_error(
                &tera,
                "Expired or invalid code while trying to log in",
                &format!(
                    "error text: {}",
                    string::String::from_utf8(bytes).unwrap_or_default()
                ),
                StatusCode::Conflict,
            )
        }
        Err(otherwise) => render_error(
            &tera,
            "Error while getting access token",
            &format!("{:?}", otherwise),
            StatusCode::InternalServerError,
        ),
    }
}

pub async fn logout(mut req: Request<State>) -> tide::Result {
    let session = req.session_mut();
    session.destroy();
    Ok(Redirect::new("/home").into())
}

pub fn gh_oauth_client(cfg: &RiteConfig) -> tide::Result<BasicClient> {
    Ok(BasicClient::new(
        ClientId::new(cfg.client_id.clone()),
        Some(ClientSecret::new(cfg.client_secret.clone())),
        AuthUrl::new(cfg.auth_url.clone())?,
        Some(TokenUrl::new(cfg.token_url.clone())?),
    )
    .set_redirect_url(RedirectUrl::new(cfg.redirect_url.clone())?))
}
