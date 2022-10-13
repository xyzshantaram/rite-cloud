use http_types::StatusCode;
use sqlx::Sqlite;
use tide_tera::{TideTeraExt, context};
use urlencoding::decode;
use tide::{Request};
use crate::{State, rite::{render_error, DocumentMetadata}};

pub async fn home(req: Request<State>) -> tide::Result {
    let state = req.state();
    let tera = state.tera.clone();
    let mut db = state.rite_db.acquire().await?;
    let username;
    if let Ok(val) = req.param("username") {
        username = decode(val)?.into_owned()
    } else {
        return render_error(tera, "Bad request.", "", StatusCode::BadRequest);
    }
    let ctx = context! {
        "section" => "blog",
        "content" => "blog for ".to_owned() + &username
    };

    let docs: Vec<DocumentMetadata> = 
        sqlx::query_as::<Sqlite, DocumentMetadata>("select * from documents where user = ? and public = 1;")
            .bind(&username)
            .fetch_all(&mut db)
            .await?;

    if docs.is_empty() {
        render_error(tera, "Blog not found", "The user whose blog you tried to view either does not exist or has no content.", StatusCode::NotFound)
    }
    else {
        tera.render_response("blog.html", &ctx)
    }
    
}