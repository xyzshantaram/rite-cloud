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
    println!("{:#?}", res);
    if res.status().is_server_error()
        || res.status().is_client_error() && res.content_type() != Some(http_types::mime::HTML)
    {
        return render_error(
            tera,
            res.status().canonical_reason(),
            "An error occurred while trying to fetch the resource you requested.",
            res.status(),
        );
    }

    Ok(res)
}
