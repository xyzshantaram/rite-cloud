use oauth2::{basic::BasicClient};

pub mod oauth_config;
pub mod auth;
pub mod routes;
pub mod middleware;

use oauth_config::OauthConfig;
use sqlx::{SqlitePool, types::chrono::NaiveDateTime};
use tide_tera::{TideTeraExt, context};

#[derive(Clone, Debug)]
pub struct State {
    pub gh_client: BasicClient,
    pub cfg: OauthConfig,
    pub tera: tera::Tera,
    pub session_db: SqlitePool,
    pub rite_db: SqlitePool
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct UploadRequest {
    pub doc: String,
    pub name: String,
    pub revision: String,
    pub token: String
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct LinkRequest {
    pub nickname: String,
    pub code: String
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct PendingClientRow {
    pub uuid: String,
    pub added_on: NaiveDateTime,
    pub user: String
}

pub fn server_error(tera: tera::Tera, title: &str, msg: &str) -> tide::Result {
    Ok(tera.render_response(
        "500.html",
        &context! {
            "section" => "error",
            "msg" => title,
            "details" => msg
        },
    )?)
}