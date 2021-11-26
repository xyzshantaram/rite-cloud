use http_types::StatusCode;
use oauth2::basic::BasicClient;

pub mod auth;
pub mod config;
pub mod middleware;
pub mod routes;

use config::RiteConfig;
use sqlx::{
    pool::PoolConnection, sqlite::SqliteRow, types::chrono::NaiveDateTime, FromRow, Row, Sqlite,
    SqlitePool,
};
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
    pub uuid: String,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct DocumentMetadata {
    pub name: String,
    pub revision: String,
    pub user: String,
    pub public: bool,
    pub uuid: String,
}

pub enum ContentGetError {
    NotFound,
    Unknown,
    Forbidden,
}

impl FromRow<'_, SqliteRow> for Client {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Client {
            added_on: row.try_get::<NaiveDateTime, &str>("added_on")?.to_string(),
            user: row.try_get("user")?,
            nickname: row.try_get("nickname")?,
            uuid: row.try_get("uuid")?,
        })
    }
}

impl FromRow<'_, SqliteRow> for Document {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Document {
            name: row.try_get("name")?,
            contents: row.try_get("contents")?,
            revision: row.try_get("revision")?,
            user: row.try_get("user")?,
            public: row.try_get("public")?,
            uuid: row.try_get("uuid")?,
        })
    }
}

impl FromRow<'_, SqliteRow> for DocumentMetadata {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(DocumentMetadata {
            name: row.try_get("name")?,
            revision: row.try_get("revision")?,
            user: row.try_get("user")?,
            public: row.try_get("public")?,
            uuid: row.try_get("uuid")?,
        })
    }
}

pub fn render_error(tera: tera::Tera, title: &str, msg: &str, status: StatusCode) -> tide::Result {
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

pub async fn contents(
    uuid: &str,
    db: &mut PoolConnection<Sqlite>,
    user: Option<String>,
) -> Result<Document, ContentGetError> {
    let doc: Document =
        sqlx::query_as::<Sqlite, Document>("select * from documents where uuid = ?;")
            .bind(uuid)
            .fetch_optional(db)
            .await
            .or(Err(ContentGetError::Unknown))?
            .ok_or(ContentGetError::NotFound)?;

    if doc.public || user == Some(doc.user.clone()) {
        Ok(doc)
    } else {
        Err(ContentGetError::Forbidden)
    }
}
