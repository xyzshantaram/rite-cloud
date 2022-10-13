use crate::{
    rite::{render_error, DocumentMetadata},
    State, TERA,
};
use http_types::StatusCode;
use sqlx::Sqlite;
use tide::Request;
use tide_tera::{context, TideTeraExt};
use urlencoding::decode;

pub async fn home(req: Request<State>) -> tide::Result {
    let state = req.state();
    let tera = TERA.clone();
    let mut db = state.rite_db.acquire().await?;
    let username;
    if let Ok(val) = req.param("username") {
        username = decode(val)?.into_owned()
    } else {
        return render_error(tera, "Bad request.", "", StatusCode::BadRequest);
    }

    let docs: Vec<DocumentMetadata> = sqlx::query_as::<Sqlite, DocumentMetadata>(
        "select * from documents where user = ? and public = 1;",
    )
    .bind(&username)
    .fetch_all(&mut db)
    .await?;

    let ctx = context! {
        "section" => "blog",
        "content" => "blog for ".to_owned() + &username,
        "docs" => docs
    };

    if docs.is_empty() {
        render_error(
            tera,
            "Blog not found",
            "The user whose blog you tried to view either does not exist or has no published content.",
            StatusCode::NotFound,
        )
    } else {
        tera.render_response("blog.html", &ctx)
    }
}
