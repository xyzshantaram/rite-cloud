use super::{render_error, Document, State};
use crate::TERA;
use http_types::StatusCode;
use sqlx::Sqlite;
use std::{borrow::Cow, collections::HashMap};
use tide::Request;
use tide_tera::{context, TideTeraExt};

pub mod api;
pub mod blog;
pub mod clients;
pub mod docs;

pub async fn homepage(req: Request<State>) -> tide::Result {
    let mut context = context! {
        "section" => "home"
    };
    let session = req.session();
    let tera = TERA.clone();
    if let Some(username) = session.get::<String>("username") {
        context.try_insert("username", &username)?;
    }
    tera.render_response("index.html", &context)
}

pub async fn error_handler(res: tide::Response) -> tide::Result {
    let tera = TERA.clone();
    if res.status().is_server_error()
        || res.status().is_client_error() && res.content_type() != Some(http_types::mime::HTML)
    {
        return render_error(
            &tera,
            res.status().canonical_reason(),
            "An error occurred while trying to fetch the resource you requested.",
            res.status(),
        );
    }

    Ok(res)
}

pub async fn confirmation_page(req: Request<State>) -> tide::Result {
    let mut params: HashMap<Cow<str>, Cow<str>> = HashMap::new();
    req.url().query_pairs().for_each(|(k, v)| {
        params.insert(k, v);
    });
    let state = req.state();
    let mut db = state.rite_db.acquire().await?;
    let tera = TERA.clone();
    let err = |e: &str| render_error(&tera, "Bad Request", e, StatusCode::BadRequest);

    let action = if let Some(val) = params.get("action") {
        val
    } else {
        return err("No action supplied.");
    };

    let doc = if let Some(val) = params.get("uuid") {
        let res = sqlx::query_as::<Sqlite, Document>("select * from documents where uuid = ?")
            .bind(val.clone().into_owned())
            .fetch_one(&mut db)
            .await;
        if let Ok(val) = res {
            val
        } else {
            return err("Invalid uuid supplied.");
        }
    } else {
        return err("No uuid supplied.");
    };

    // ok to unwrap here because we are already logged in, thanks to the magic of middleware
    let username = req.session().get::<String>("username").unwrap();

    let context = context! {
        "section" => "confirm",
        "username" => username,
        "doc" => doc,
        "action" => action
    };

    tera.render_response("confirm.html", &context)
}
