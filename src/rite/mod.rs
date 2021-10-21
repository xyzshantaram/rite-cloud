use http_types::StatusCode;
use oauth2::basic::BasicClient;

pub mod auth;
pub mod middleware;
pub mod config;
pub mod routes;

use config::RiteConfig;
use sqlx::{sqlite::SqliteRow, types::chrono::NaiveDateTime, FromRow, Row, SqlitePool};
use tide_tera::{context, TideTeraExt};

#[derive(Clone, Debug)]
pub struct State {
    pub gh_client: BasicClient,
    pub cfg: RiteConfig,
    pub tera: tera::Tera,
    pub session_db: SqlitePool,
    pub rite_db: SqlitePool,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct LinkRequest {
    pub nickname: String,
    pub token: String,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Client {
    pub added_on: String,
    pub user: String,
    pub nickname: String,
    pub uuid: String,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Document {
    pub name: String,
    pub revision: String,
    pub contents: String,
    pub user: String,
    pub public: bool,
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

impl FromRow<'_, SqliteRow> for Document {
    fn from_row<'r>(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let mut rv = Document::default();
        rv.name = row.try_get("name")?;
        rv.contents = row.try_get("contents")?;
        rv.revision = row.try_get("revision")?;
        rv.user = row.try_get("user")?;
        rv.public = if row.try_get("public")? { true } else { false };
        Ok(rv)
    }
}

pub fn server_error(tera: tera::Tera, title: &str, msg: &str, status: StatusCode) -> tide::Result {
    let mut res = tera.render_response(
        "500.html",
        &context! {
            "section" => "error",
            "msg" => title,
            "details" => msg
        },
    )?;
    res.set_status(status);

    Ok(res)
}
