use crate::{
    rite::{render_error, DocumentMetadata},
    State, TERA,
};
use http_types::StatusCode;
use sqlx::Sqlite;
use tide::Request;
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
        "section" => "blog",
        "content" => "blog for ".to_owned() + &author,
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
    let username = session.get::<String>("username");
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
