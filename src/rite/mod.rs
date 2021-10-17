use oauth2::{basic::BasicClient};

pub mod oauth_config;
pub mod auth;
pub mod routes;
pub mod middleware;

use oauth_config::OauthConfig;
use sqlx::{FromRow, Row, SqlitePool, sqlite::SqliteRow, types::chrono::NaiveDateTime};
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
    pub token: String
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Client {
    pub added_on: String,
    pub user: String,
    pub nickname: String,
    pub uuid: String
}

impl FromRow<'_, SqliteRow> for Client {
    fn from_row<'r>(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let mut rv = Client::default();
        rv.added_on = row.try_get::<NaiveDateTime, &str>("added_on")?.to_string();
        rv.user = row.try_get("user")?;
        rv.nickname = row.try_get("nickname")?;
        rv.uuid = row.try_get("uuid")?;
        Ok(rv)
    }
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