use crate::{
    rite::{render_error, Document},
    State,
};
use http_types::{convert::json, mime, StatusCode};
use sqlx::{query::QueryAs, sqlite::SqliteArguments, Sqlite};
use tide::{Redirect, Request, Response};
use tide_tera::{context, TideTeraExt};
use urlencoding::decode;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct UploadRequest {
    pub name: String,
    pub revision: String,
    pub contents: String,
    pub token: String,
    pub user: String,
    pub public: bool,
}

pub async fn api_upload_doc(mut req: Request<State>) -> tide::Result {
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
        res.set_status(StatusCode::Conflict);
        res.set_body(json!({
            "message": "Duplicate revision."
        }));
    }

    Ok(res)
}

pub async fn delete(req: Request<State>) -> tide::Result {
    let state = req.state();
    let tera = state.tera.clone();
    let mut db = state.rite_db.acquire().await?;

    let username: String = req.session().get("username").unwrap();
    // unwrapping this is ok because we have the login middleware

    let doc_ = req.param("name");
    if doc_.is_err() {
        return render_error(tera, "Bad request.", "", StatusCode::BadRequest);
    }

    let doc = decode(doc_?)?.into_owned();

    let revision = if let Ok(rev) = req.param("revision") {
        Some(decode(rev)?.into_owned())
    } else {
        None
    };

    let mut doc_query: QueryAs<Sqlite, Document, SqliteArguments> =
        sqlx::query_as::<Sqlite, Document>(if revision.is_some() {
            "select * from documents where user = ? and name = ? and revision = ?;"
        } else {
            "select * from documents where user = ? and name = ?;"
        })
        .bind(&username)
        .bind(&doc);

    if let Some(val) = revision.clone() {
        doc_query = doc_query.bind(val);
    }

    let doc_exists: Option<Document> = doc_query.fetch_optional(&mut db).await?;

    if doc_exists.is_some() {
        let mut query = sqlx::query(if revision.is_some() {
            "delete from documents where user = ? and name = ? and revision = ?;"
        } else {
            "delete from documents where user = ? and name = ?;"
        })
        .bind(username)
        .bind(doc);
        if revision.is_some() {
            query = query.bind(revision.unwrap());
        }
        query.execute(&mut db).await?;
        Ok(Redirect::new("/docs/list").into())
    } else {
        render_error(
            tera,
            "Not found.",
            "The document specified for deletion was not found.",
            StatusCode::NotFound,
        )
    }
}

pub async fn view(mut req: Request<State>) -> tide::Result {
    unimplemented!()
}

pub async fn api_list_docs(mut req: Request<State>) -> tide::Result {
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
