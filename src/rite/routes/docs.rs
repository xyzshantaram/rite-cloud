use crate::{
    rite::{render_error, Document},
    State,
};
use http_types::{convert::json, mime, StatusCode};
use sqlx::Sqlite;
use tide::{Request, Response};
use tide_tera::{context, TideTeraExt};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct UploadRequest {
    pub name: String,
    pub revision: String,
    pub contents: String,
    pub token: String,
    pub user: String,
    pub public: bool,
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

    if doc.is_none() {
        let public = if body.public { 1 } else { 0 };
        sqlx::query("insert into documents(name, user, revision, contents, public, added_on) values(?, ?, ?, ?, ?, datetime('now'));")
            .bind(&body.name)
            .bind(&body.user)
            .bind(&body.revision)
            .bind(&body.contents)
            .bind(public)
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

pub async fn list(req: Request<State>) -> tide::Result {
    let state = req.state();
    let session = req.session();
    let mut db = state.rite_db.acquire().await?;
    let tera = state.tera.clone();

    if let Some(val) = session.get::<String>("username") {
        let username = val;
        let rows: Vec<Document> = sqlx::query_as::<Sqlite, Document>(
            "SELECT name, contents, revision, user, public from documents where user = ?;",
        )
        .bind(&username)
        .fetch_all(&mut db)
        .await?;

        let mut context = context! {
            "section" => "view documents"
        };
        context.try_insert("docs", &rows)?;
        context.try_insert("username", &username)?;

        tera.render_response("view_documents.html", &context)
    } else {
        render_error(tera, "Unknown error.", "", StatusCode::InternalServerError)
    }
}
