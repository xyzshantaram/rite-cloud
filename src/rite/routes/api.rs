use http_types::{mime, StatusCode};
use serde_json::json;
use sqlx::Sqlite;
use tide::{Request, Response};
use uuid::Uuid;

use crate::rite::{ContentGetError, DocumentMetadata, State};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct UploadRequest {
    pub name: String,
    pub revision: String,
    pub contents: String,
    pub token: String,
    pub user: String,
    pub public: bool,
    pub encrypted: Option<bool>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct BasicClientRequest {
    pub token: String,
    pub user: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ContentRequest {
    pub token: String,
    pub user: String,
    pub uuid: String,
}

pub async fn list(mut req: Request<State>) -> tide::Result {
    let state = req.state();
    let mut db = state.rite_db.acquire().await?;
    let body: BasicClientRequest = req.body_json().await?;

    let rows: Vec<DocumentMetadata> =
        sqlx::query_as::<Sqlite, DocumentMetadata>("SELECT * from documents where user = ?;")
            .bind(&body.user)
            .fetch_all(&mut db)
            .await?;

    let mut res = Response::new(StatusCode::Ok);
    res.set_content_type(mime::JSON);
    res.insert_header("Access-Control-Allow-Origin", "*");

    if let Ok(val) = serde_json::to_value(rows) {
        res.set_body(val);
    } else {
        res.set_status(StatusCode::InternalServerError);
        res.set_body(json!({
            "message": "Unknown error."
        }));
    }

    Ok(res)
}

pub async fn contents(mut req: Request<State>) -> tide::Result {
    let json: ContentRequest = req.body_json().await?;
    let state = req.state();
    let mut db = state.rite_db.acquire().await?;

    let mut res = Response::new(StatusCode::Ok);
    res.insert_header("Access-Control-Allow-Origin", "*");

    match crate::rite::contents(&json.uuid, &mut db, Some(json.user)).await {
        Ok(doc) => {
            let encrypted = doc.encrypted.unwrap_or(false);
            res.set_body(
                json!({ "message": "Ok", "contents": doc.contents, "encrypted": encrypted }),
            )
        }
        Err(kind) => match kind {
            ContentGetError::NotFound => {
                res.set_status(StatusCode::NotFound);
                res.set_body(json!({
                    "message": "Not found."
                }));
            }
            ContentGetError::Forbidden => {
                res.set_status(StatusCode::Forbidden);
                res.set_body(json!({
                    "message": "Forbidden."
                }))
            }
            ContentGetError::Unknown => {
                res.set_status(StatusCode::InternalServerError);
                res.set_body(json!({
                    "message": "An unknown error occurred."
                }))
            }
        },
    }
    Ok(res)
}

pub async fn upload(mut req: Request<State>) -> tide::Result {
    let body: UploadRequest = req.body_json().await?;
    let state = req.state();
    let mut db = state.rite_db.acquire().await?;

    let doc = sqlx::query("select * from documents where user = ? and name = ? and revision = ?;")
        .bind(&body.user)
        .bind(&body.name)
        .bind(&body.revision)
        .fetch_optional(&mut db)
        .await?;

    let mut res = Response::new(StatusCode::Ok);
    res.set_content_type(mime::JSON);
    res.insert_header("Access-Control-Allow-Origin", "*");
    let uuid = Uuid::new_v4();

    if doc.is_none() {
        sqlx::query("insert into documents(name, user, revision, contents, public, added_on, uuid, encrypted) values(?, ?, ?, ?, ?, datetime('now'), ?, ?);")
            .bind(&body.name)
            .bind(&body.user)
            .bind(&body.revision)
            .bind(&body.contents)
            .bind(if body.public { 1 } else { 0 })
            .bind(uuid.to_string())
            .bind(if body.encrypted.unwrap_or(false) { 1 } else { 0 })
            .execute(&mut db)
            .await?;
        res.set_body(json!({ "message": "Ok", "uuid": uuid.to_string() }));
    } else {
        res.set_status(StatusCode::Conflict);
        res.set_body(json!({ "message": "Duplicate revision." }));
    }

    Ok(res)
}
