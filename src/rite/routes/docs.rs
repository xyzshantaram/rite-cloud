use crate::{
    rite::{render_error, Document},
    State,
};
use http_types::{convert::json, headers::ACCESS_CONTROL_ALLOW_ORIGIN, mime, StatusCode};
use serde::Deserialize;
use sqlx::{pool::PoolConnection, query::QueryAs, sqlite::SqliteArguments, Sqlite};
use tide::{Redirect, Request, Response};
use tide_tera::{context, TideTeraExt};
use urlencoding::decode;
use uuid::Uuid;

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
    res.insert_header("Access-Control-Allow-Origin", "*");
    let uuid = Uuid::new_v4();

    if doc.is_none() {
        let public = if body.public { 1 } else { 0 };
        sqlx::query("insert into documents(name, user, revision, contents, public, added_on, uuid) values(?, ?, ?, ?, ?, datetime('now'), ?);")
            .bind(&body.name)
            .bind(&body.user)
            .bind(&body.revision)
            .bind(&body.contents)
            .bind(public)
            .bind(uuid.to_string())
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

    let doc;
    if let Ok(val) = req.param("name") {
        doc = decode(val)?.into_owned()
    } else {
        return render_error(tera, "Bad request.", "", StatusCode::BadRequest);
    }

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

pub enum ContentGetError {
    NotFound,
    QueryFailed,
    Forbidden,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct ViewQueryParams {
    raw: bool,
}

impl Default for ViewQueryParams {
    fn default() -> Self {
        Self { raw: false }
    }
}

pub async fn view(req: Request<State>) -> tide::Result {
    let session = req.session();
    let req_uuid = req.param("uuid")?;
    let uuid = decode(req_uuid)?;

    let loggedin = session.get::<String>("username").is_some();
    let params: ViewQueryParams = req.query()?;
    let state = req.state().clone();
    let mut db = state.rite_db.acquire().await?;
    let tera = state.tera.clone();

    match contents(&uuid, &mut db, !loggedin).await {
        Ok(val) => {
            if params.raw {
                let mut res = Response::new(StatusCode::Ok);
                res.set_content_type(mime::PLAIN);
                res.set_body(val);
                Ok(res)
            } else {
                let ctx = context! {
                    "username" => session.get::<String>("username").unwrap_or_default(),
                    "section" => "view document",
                    "contents" => val
                };

                tera.render_response("view_document.html", &ctx)
            }
        }
        Err(kind) => match kind {
            ContentGetError::NotFound => render_error(
                tera,
                "Not Found",
                "The document you requested was not found.",
                StatusCode::NotFound,
            ),
            ContentGetError::Forbidden => render_error(
                tera,
                "Forbidden",
                "You are not authorized to view the requested document.",
                StatusCode::Forbidden,
            ),
            ContentGetError::QueryFailed => render_error(
                tera,
                "Error",
                "An unknown error occurred.",
                StatusCode::InternalServerError,
            ),
        },
    }
}

pub async fn api_list_docs(mut req: Request<State>) -> tide::Result {
    unimplemented!()
}

pub async fn api_contents(mut req: Request<State>) -> tide::Result {
    unimplemented!()
}

pub async fn toggle_visibility(mut req: Request<State>) -> tide::Result {
    unimplemented!()
}

pub async fn contents(
    uuid: &str,
    db: &mut PoolConnection<Sqlite>,
    public_access: bool,
) -> Result<String, ContentGetError> {
    let query =
        sqlx::query_as::<Sqlite, Document>("select * from documents where uuid = ?;").bind(uuid);
    let res_: Result<Option<Document>, sqlx::Error> = query.fetch_optional(db).await;
    if let Ok(res) = res_ {
        if let Some(doc) = res {
            if public_access && !doc.public {
                Err(ContentGetError::Forbidden)
            } else {
                Ok(doc.contents)
            }
        } else {
            Err(ContentGetError::NotFound)
        }
    } else {
        Err(ContentGetError::QueryFailed)
    }
}

pub async fn list(req: Request<State>) -> tide::Result {
    let state = req.state();
    let session = req.session();
    let mut db = state.rite_db.acquire().await?;
    let tera = state.tera.clone();

    if let Some(val) = session.get::<String>("username") {
        let username = val;
        let rows: Vec<Document> = sqlx::query_as::<Sqlite, Document>(
            "SELECT name, contents, revision, user, public, uuid from documents where user = ?;",
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
