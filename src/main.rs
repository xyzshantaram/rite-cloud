mod rite;

use rite::{State, auth::{self, logout}, oauth_config::OauthConfig};
use surf::http::cookies::SameSite;
use tera::{Context, Tera};
use tide::{Request};
use tide_tera::prelude::*;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut cfg = OauthConfig::default();
    if let Err(e) = cfg.fill_from(std::env::var) {
        panic!("Error getting environment variable {}: {}", e.0, e.1);
    }

    let tera: Tera = {
        let mut tera =
            Tera::new("templates/**/*").expect("Error parsing templates while initialising Tera.");
        tera.autoescape_on(vec!["html"]);
        tera
    };

    let state = State {
        gh_client: auth::gh_oauth_client(&cfg).unwrap(),
        cfg: cfg.clone(),
        tera
    };

    let mut app = tide::with_state(state);

    app.with(
        tide::sessions::SessionMiddleware::new(
            tide::sessions::MemoryStore::new(),
            cfg.tide_secret.as_bytes(),
        )
        .with_same_site_policy(SameSite::Lax),
    );

    app.at("/").get(homepage);

    app.at("/auth/github").get(auth::gh);
    app.at("/auth/logout").get(logout);
    app.at("/auth/github/authorized").get(auth::gh_authorized);

    app.listen(cfg.app_url).await?;
    Ok(())
}

async fn homepage(req: Request<State>) -> tide::Result {
    let mut context = Context::new();
    let session = req.session();
    let tera = req.state().tera.clone();
    match session.get::<String>("username") {
        Some(val) => context.insert("username", &val),
        _ => {}
    }

    Ok(tera.render_response("index.html", &context)?.into())
}