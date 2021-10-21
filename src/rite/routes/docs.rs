use crate::State;
use http_types::{convert::json, mime, StatusCode};
use sqlx::{Pool, Sqlite};
use tide::{Request, Response};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct UploadRequest {
    pub name: String,
    pub revision: String,
    pub contents: String,
    pub token: String,
    pub user: String,
    pub visibility: bool,
}
pub async fn upload(mut req: Request<State>) -> tide::Result {
    tide::log::info!("Made it here");
    let body: UploadRequest = req.body_json().await?;
    let state = req.state();
    let mut db = state.rite_db.acquire().await?;

    tide::log::info!("Here");

    let doc = sqlx::query("select * from documents where user = ? and name = ? and revision = ?;")
        .bind(&body.user)
        .bind(&body.name)
        .bind(&body.revision)
        .fetch_optional(&mut db)
        .await?;

    let mut res = Response::new(StatusCode::Ok);
    if let None = doc {
        let visibility = if body.visibility { 1 } else { 0 };
        sqlx::query("insert into documents(name, user, revision, contents, visibility) values(?, ?, ?, ?, ?);")
            .bind(&body.name)
            .bind(&body.user)
            .bind(&body.revision)
            .bind(&body.contents)
            .bind(visibility)
            .execute(&mut db)
            .await?;
        res.set_body(json!({
            "message": "Ok"
        }));
    } else {
        res.set_content_type(mime::JSON);
        res.set_status(StatusCode::Conflict);
        res.set_body(json!({
            "message": "Duplicate revision."
        }));
    }

    Ok(res)
}

pub async fn delete(mut req: Request<State>) -> tide::Result {
    unimplemented!()
}

pub async fn view(mut req: Request<State>) -> tide::Result {
    unimplemented!()
}

pub async fn clist(mut req: Request<State>) -> tide::Result {
    unimplemented!()
}

pub async fn list(mut req: Request<State>) -> tide::Result {
    unimplemented!()
}
