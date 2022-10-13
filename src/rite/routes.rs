use http_types::{StatusCode};
use tide_tera::{context, TideTeraExt};

use crate::TERA;

use super::{render_error, State};

pub mod api;
pub mod blog;
pub mod clients;
pub mod docs;

use tide::Request;

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
    if let Some(err) = res.error() {
        render_error(
            tera,
            "Error",
            err.type_name().get_or_insert("An error occurred."),
            res.status(),
        )
    } else {
        render_error(
            tera,
            "Error",
            "An error occurred.",
            StatusCode::InternalServerError,
        )
    }
}
