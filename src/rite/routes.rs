use tide_tera::{context, TideTeraExt};

use super::State;

pub mod clients;
pub mod docs;

use tide::Request;

pub async fn homepage(req: Request<State>) -> tide::Result {
    let mut context = context! {
        "section" => "home"
    };
    let session = req.session();
    let tera = req.state().tera.clone();
    if let Some(username) = session.get::<String>("username") {
        context.try_insert("username", &username)?;
    }
    Ok(tera.render_response("index.html", &context)?)
}
