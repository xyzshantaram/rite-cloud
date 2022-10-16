use crate::{
    rite::{render_error, DocumentMetadata},
    State, TERA,
};
use http_types::StatusCode;
use serde::Deserialize;
use sqlx::Sqlite;
use tide::{Redirect, Request};
use tide_tera::{context, TideTeraExt};
use urlencoding::decode;

use super::docs::group_revisions_by_doc;

pub async fn home(req: Request<State>) -> tide::Result {
    let state = req.state();
    let tera = TERA.clone();
    let mut db = state.rite_db.acquire().await?;
    let author = if let Ok(val) = req.param("author") {
        decode(val)?.into_owned()
    } else {
        return render_error(&tera, "Bad request.", "", StatusCode::BadRequest);
    };

    let session = req.session();
    let username = session.get::<String>("username");

    let docs: Vec<DocumentMetadata> = sqlx::query_as::<Sqlite, DocumentMetadata>(
        "select * from documents where user = ? and published_title is not null;",
    )
    .bind(&author)
    .fetch_all(&mut db)
    .await?;

    let ctx = context! {
        "title" => author.clone() + "'s blog",
        "section" => author.clone() + "'s blog",
        "author" => &author,
        "docs" => docs,
        "username" => username
    };

    if docs.is_empty() {
        render_error(
            &tera,
            "Blog not found",
            "The user whose blog you tried to view either does not exist or has no published content.",
            StatusCode::NotFound,
        )
    } else {
        tera.render_response("blog.html", &ctx)
    }
}

pub async fn manage(req: Request<State>) -> tide::Result {
    let state = req.state();
    let tera = TERA.clone();

    let session = req.session();
    let username = session.get::<String>("username").unwrap();
    let mut db = state.rite_db.acquire().await?;

    let mut docs: Vec<DocumentMetadata> =
        sqlx::query_as::<Sqlite, DocumentMetadata>("select * from documents where user = ?")
            .bind(&username)
            .fetch_all(&mut db)
            .await?;
    docs.sort_by_key(|e| e.name.to_lowercase());

    let grouped = group_revisions_by_doc(docs.as_slice());

    let ctx = context! {
        "username" => username,
        "docs" => grouped,
        "section" => "manage your blog"
    };

    tera.render_response("manage_blog.html", &ctx)
}

#[derive(Deserialize)]
pub struct PublishRequest {
    uuid: String,
    publish_title: String,
}

#[derive(Deserialize)]
pub struct UnpublishRequest {
    uuid: String,
}

pub async fn publish(mut req: Request<State>) -> tide::Result {
    let body: PublishRequest = req.body_form().await?;
    let session = req.session();
    let username = session.get::<String>("username").unwrap();
    let state = req.state();
    let mut db = state.rite_db.acquire().await?;

    if let Some(val) = sqlx::query_as::<Sqlite, DocumentMetadata>(
        "select * from documents where user = ? and uuid = ?",
    )
    .bind(&username)
    .bind(body.uuid)
    .fetch_optional(&mut db)
    .await?
    {
        sqlx::query(
            "update documents set published_title = ?, publish_date = datetime('now') where user = ? and uuid = ?"
        )
            .bind(body.publish_title)    
            .bind(&username)
            .bind(val.uuid)
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
    Ok(Redirect::new("/blog/manage").into())
}

pub async fn unpublish(mut req: Request<State>) -> tide::Result {
    let body: UnpublishRequest = req.body_form().await?;
    let session = req.session();
    let username = session.get::<String>("username").unwrap();
    let state = req.state();
    let mut db = state.rite_db.acquire().await?;

    if let Some(val) = sqlx::query_as::<Sqlite, DocumentMetadata>(
        "select * from documents where user = ? and uuid = ?",
    )
    .bind(&username)
    .bind(body.uuid)
    .fetch_optional(&mut db)
    .await?
    {
        sqlx::query(
            "update documents set published_title = NULL, publish_date = NULL where user = ? and uuid = ?"
        )
            .bind(&username)
            .bind(val.uuid)
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
    Ok(Redirect::new("/blog/manage").into())
}
