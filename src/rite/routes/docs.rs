use crate::{
    rite::{contents, render_error, ContentGetError, Document, DocumentMetadata},
    State, TERA,
};
use http_types::{mime, StatusCode};
use indexmap::IndexMap;
use serde::Deserialize;
use sqlx::Sqlite;
use tide::{Redirect, Request, Response};
use tide_tera::{context, TideTeraExt};
use urlencoding::decode;

#[derive(Deserialize)]
pub struct MutateRequestBody {
    uuid: String,
}

pub async fn delete(mut req: Request<State>) -> tide::Result {
    let state = req.state();
    let mut db = state.rite_db.acquire().await?;

    let username: String = req.session().get("username").unwrap();
    // unwrapping this is ok because we have the login middleware
    let body: MutateRequestBody = req.body_form().await?;

    sqlx::query("delete from documents where user = ? and uuid = ?")
        .bind(username)
        .bind(body.uuid)
        .execute(&mut db)
        .await?;
    Ok(Redirect::new("/docs/list").into())
}

pub async fn delete_all(mut req: Request<State>) -> tide::Result {
    let state = req.state();
    let mut db = state.rite_db.acquire().await?;

    let username: String = req.session().get("username").unwrap();
    // unwrapping this is ok because we have the login middleware
    let body: MutateRequestBody = req.body_form().await?;

    if let Some(val) = sqlx::query_as::<Sqlite, DocumentMetadata>(
        "select * from documents where user = ? and uuid = ?",
    )
    .bind(&username)
    .bind(body.uuid)
    .fetch_optional(&mut db)
    .await?
    {
        sqlx::query("delete from documents where user = ? and name = ?")
            .bind(&username)
            .bind(val.name)
            .execute(&mut db)
            .await?;
    } else {
        return render_error(
            &TERA.clone(),
            "Bad request.",
            "Invalid uuid supplied.",
            StatusCode::BadRequest,
        );
    };

    Ok(Redirect::new("/docs/list").into())
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct ViewQueryParams {
    raw: bool,
}

pub async fn view(req: Request<State>) -> tide::Result {
    let session = req.session();
    let req_uuid = req.param("uuid")?;
    let uuid = decode(req_uuid)?;

    let username = session.get::<String>("username");
    let params: ViewQueryParams = req.query()?;
    let state = req.state().clone();
    let mut db = state.rite_db.acquire().await?;
    let tera = TERA.clone();

    match contents(&uuid, &mut db, username.clone()).await {
        Ok(doc) => {
            if params.raw {
                let mut res = Response::new(StatusCode::Ok);
                res.set_content_type(mime::PLAIN);
                res.set_body(doc.contents);
                Ok(res)
            } else {
                let encrypted = doc.encrypted.unwrap_or(false);
                let ctx = context! {
                    "username" => username.unwrap_or_default(),
                    "section" => "view document",
                    "contents" => doc.contents,
                    "title" => doc.name,
                    "revision" => doc.revision,
                    "encrypted" => encrypted
                };

                tera.render_response("view_document.html", &ctx)
            }
        }
        Err(kind) => match kind {
            ContentGetError::NotFound => render_error(
                &tera,
                "Not Found",
                "The document you requested was not found.",
                StatusCode::NotFound,
            ),
            ContentGetError::Forbidden => render_error(
                &tera,
                "Forbidden",
                "You are not authorized to view the requested document. Ask the owner to make it public. If you are the owner, log in to view this document.",
                StatusCode::Forbidden,
            ),
            ContentGetError::Unknown => render_error(
                &tera,
                "Error",
                "An unknown error occurred.",
                StatusCode::InternalServerError,
            ),
        },
    }
}

pub async fn toggle_visibility(mut req: Request<State>) -> tide::Result {
    let state = req.state();
    let tera = TERA.clone();
    let mut db = state.rite_db.acquire().await?;
    let username: String = req.session().get("username").unwrap();
    let body: MutateRequestBody = req.body_form().await?;

    let doc_exists: Option<Document> =
        sqlx::query_as::<Sqlite, Document>("select * from documents where user = ? and uuid = ?;")
            .bind(&username)
            .bind(&body.uuid)
            .fetch_optional(&mut db)
            .await?;

    if doc_exists.is_some() {
        sqlx::query("update documents set public = ((public | 1) - (public & 1)) where user = ? and uuid = ?;")
        .bind(&username)    
        .bind(&body.uuid)
            .execute(&mut db)
            .await?;
        Ok(Redirect::new("/docs/list").into())
    } else {
        render_error(
            &tera,
            "Not found.",
            "The specified document was not found.",
            StatusCode::NotFound,
        )
    }
}

pub fn group_revisions_by_doc(
    revisions: &[DocumentMetadata],
) -> IndexMap<String, Vec<DocumentMetadata>> {
    let mut map: IndexMap<String, Vec<DocumentMetadata>> = IndexMap::new();
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
    let tera = TERA.clone();

    if let Some(val) = session.get::<String>("username") {
        let username = val;
        let mut rows: Vec<DocumentMetadata> =
            sqlx::query_as::<Sqlite, DocumentMetadata>("SELECT * from documents where user = ?;")
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
        render_error(&tera, "Unknown error.", "", StatusCode::InternalServerError)
    }
}
