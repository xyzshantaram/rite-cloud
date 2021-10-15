use std::string;

use oauth2::basic::BasicClient;
use oauth2::curl::http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, RequestTokenError,
    Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use tide_tera::{context, TideTeraExt};

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

#[derive(Deserialize, Serialize)]
struct GhResponse {
    login: String,
}

use super::OauthConfig;
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
    println!("45 {} {}", query.code, query.state);
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
            session.insert("username", res.login)?;
            return Ok(Redirect::new("/").into());
        }
        Err(RequestTokenError::Parse(_, bytes)) => {
            return tera.render_response("500.html", &context! {
                "msg" => "Expired or invalid response",
                "details" => format!("error text: {}", string::String::from_utf8(bytes).unwrap_or_default())
            });
        }
        Err(otherwise) => {
            return tera.render_response(
                "500.html",
                &context! {
                    "msg" => "Error while getting access token",
                    "details" => format!("{:?}", otherwise)
                },
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
