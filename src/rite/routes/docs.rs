use crate::{
    rite::{contents, render_error, ContentGetError, Document},
    State,
};
use http_types::{mime, StatusCode};
use indexmap::IndexMap;
use serde::Deserialize;
use sqlx::{query::QueryAs, sqlite::SqliteArguments, Sqlite};
use tide::{Redirect, Request, Response};
use tide_tera::{context, TideTeraExt};
use urlencoding::decode;

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

    let username = session.get::<String>("username");
    let params: ViewQueryParams = req.query()?;
    let state = req.state().clone();
    let mut db = state.rite_db.acquire().await?;
    let tera = state.tera.clone();

    match contents(&uuid, &mut db, username.clone()).await {
        Ok(doc) => {
            if params.raw {
                let mut res = Response::new(StatusCode::Ok);
                res.set_content_type(mime::PLAIN);
                res.set_body(doc.contents);
                Ok(res)
            } else {
                let ctx = context! {
                    "username" => username.unwrap_or_default(),
                    "section" => "view document",
                    "contents" => doc.contents,
                    "title" => doc.name,
                    "revision" => doc.revision
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
                "You are not authorized to view the requested document. Ask the owner to make it public. If you are the owner, log in to view this document.",
                StatusCode::Forbidden,
            ),
            ContentGetError::Unknown => render_error(
                tera,
                "Error",
                "An unknown error occurred.",
                StatusCode::InternalServerError,
            ),
        },
    }
}

pub async fn toggle_visibility(req: Request<State>) -> tide::Result {
    let state = req.state();
    let tera = state.tera.clone();
    let mut db = state.rite_db.acquire().await?;

    let uuid;
    if let Ok(val) = req.param("uuid") {
        uuid = decode(val)?.into_owned()
    } else {
        return render_error(tera, "Bad request.", "", StatusCode::BadRequest);
    }

    let doc_exists: Option<Document> =
        sqlx::query_as::<Sqlite, Document>("select * from documents where uuid = ?;")
            .bind(&uuid)
            .fetch_optional(&mut db)
            .await?;

    if doc_exists.is_some() {
        sqlx::query("update documents set public = ((public | 1) - (public & 1)) where uuid = ?;")
            .bind(uuid)
            .execute(&mut db)
            .await?;
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

pub fn group_revisions_by_doc(revisions: &[Document]) -> IndexMap<String, Vec<Document>> {
    let mut map: IndexMap<String, Vec<Document>> = IndexMap::new();
    revisions.iter().for_each(|elem| {
        let clone = elem.clone();
        if map.contains_key(&elem.name) {
            map.get_mut(&elem.name).unwrap().push(clone);
        } else {
            map.insert(elem.name.clone(), vec![clone]);
        }
    });

    map
}

pub async fn list(req: Request<State>) -> tide::Result {
    let state = req.state();
    let session = req.session();
    let mut db = state.rite_db.acquire().await?;
    let tera = state.tera.clone();

    if let Some(val) = session.get::<String>("username") {
        let username = val;
        let mut rows: Vec<Document> = sqlx::query_as::<Sqlite, Document>(
            "SELECT name, contents, revision, user, public, uuid from documents where user = ?;",
        )
        .bind(&username)
        .fetch_all(&mut db)
        .await?;

        let mut context = context! {
            "section" => "view documents"
        };

        rows.sort_by_key(|e| e.name.to_lowercase());
        let docs = group_revisions_by_doc(&rows);

        context.try_insert("docs", &docs)?;
        context.try_insert("username", &username)?;

        tera.render_response("view_documents.html", &context)
    } else {
        render_error(tera, "Unknown error.", "", StatusCode::InternalServerError)
    }
}
