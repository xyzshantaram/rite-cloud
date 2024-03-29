mod rite;

use std::time::Duration;

use async_sqlx_session::SqliteSessionStore;
use http_types::cookies::SameSite;
use lazy_static::lazy_static;
use rite::{
    auth::{self, logout},
    config::RiteConfig,
    middleware::{DocPrelimChecks, WebAuthCheck},
    routes, State,
};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tera::Tera;
use tide::{sessions::SessionMiddleware, utils::After};
use tide_governor::GovernorMiddleware;

lazy_static! {
    pub static ref TERA: Tera = {
        let mut tera =
            Tera::new("templates/**/*").expect("Error parsing templates while initialising Tera.");
        tera.autoescape_on(vec!["html", "sql"]);
        tera
    };
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut cfg = RiteConfig::default();
    if let Err(e) = cfg.fill_from(std::env::var) {
        panic!("Error getting environment variable {}: {}", e.0, e.1);
    }

    let session_db = db_connection(cfg.session_db_url.clone()).await?;
    let mut rite_db = db_connection(cfg.rite_db_url.clone()).await?;
    initialise_db(&mut rite_db).await?;

    let state = State {
        gh_client: auth::gh_oauth_client(&cfg).unwrap(),
        cfg: cfg.clone(),
        session_db: session_db.clone(),
        rite_db,
    };

    let mut app = tide::with_state(state.clone());

    app.with(tide::log::LogMiddleware::new());
    app.with(build_session_middleware(session_db, &cfg.tide_secret).await?);

    if std::env::var("RITE_LOG").is_ok() {
        tide::log::start();
    }

    // assets
    app.at("/res").serve_dir("res")?;

    // static pages
    app.at("/").serve_file("res/landing.html")?;
    app.at("/help/").serve_dir("res/docs")?;
    app.at("/help").serve_file("res/docs/index.html")?;
    app.at("/help/").serve_file("res/docs/index.html")?;

    // homepage
    app.at("/home").get(routes::homepage);

    let auth = {
        let mut app = tide::with_state(state.clone());
        app.at("/github").get(auth::gh);
        app.at("/github/authorized").get(auth::gh_authorized);

        app.at("/logout").get(logout);
        app
    };
    let clients = {
        let mut app = tide::with_state(state.clone());
        app.with(WebAuthCheck::new());

        app.at("/link").get(routes::clients::link_get);
        app.at("/view").get(routes::clients::view);
        app.at("/create").post(routes::clients::create);
        app.at("/delete/:uuid").get(routes::clients::delete);
        app
    };

    let docs = {
        let mut app = tide::with_state(state.clone());

        app.at("/view/:uuid").get(routes::docs::view);
        app.at("/list")
            .with(WebAuthCheck::new())
            .get(routes::docs::list);
        app.at("/delete")
            .with(WebAuthCheck::new())
            .post(routes::docs::delete);
        app.at("/delete-all")
            .with(WebAuthCheck::new())
            .post(routes::docs::delete_all);
        app.at("/toggle-visibility")
            .with(WebAuthCheck::new())
            .post(routes::docs::toggle_visibility);

        app
    };

    let api = {
        let mut app = tide::with_state(state.clone());
        app.at("/docs/upload")
            .with(GovernorMiddleware::per_minute(2)?)
            .with(DocPrelimChecks::new())
            .post(routes::api::upload);
        app.at("/docs/list")
            .with(DocPrelimChecks::new())
            .post(routes::api::list);

        app.at("/docs/contents").post(routes::api::contents);
        app
    };

    app.at("/blog/").serve_file("res/blog-tutorial.html")?;
    let blog = {
        let mut app = tide::with_state(state);
        app.at("/").serve_file("res/blog-tutorial.html")?;
        app.at("/:author").get(routes::blog::home);
        app.at("/manage")
            .with(WebAuthCheck::new())
            .get(routes::blog::manage);
        app.at("/publish")
            .with(WebAuthCheck::new())
            .post(routes::blog::publish);
        app.at("/unpublish")
            .with(WebAuthCheck::new())
            .post(routes::blog::unpublish);
        app
    };

    app.at("/clients").nest(clients);
    app.at("/docs").nest(docs);
    app.at("/auth").nest(auth);
    app.at("/api").nest(api);
    app.at("/blog").nest(blog);
    app.at("/confirm").get(routes::confirmation_page);

    app.with(After(routes::error_handler));
    app.listen(cfg.app_url).await?;
    Ok(())
}

async fn db_connection(url: String) -> tide::Result<SqlitePool> {
    Ok(SqlitePoolOptions::new().connect(&url).await?)
}

async fn build_session_middleware(
    db: SqlitePool,
    secret: &str,
) -> tide::Result<SessionMiddleware<SqliteSessionStore>> {
    let session_store = SqliteSessionStore::from_client(db);
    session_store.migrate().await?;
    session_store.spawn_cleanup_task(Duration::from_secs(60 * 15));
    let session_secret = secret;
    Ok(
        SessionMiddleware::new(session_store, session_secret.as_bytes())
            .with_same_site_policy(SameSite::Lax),
    )
}

async fn initialise_db(db: &mut SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS clients (
        uuid TEXT UNIQUE,
        token TEXT,
        user TEXT,
        nickname TEXT,
        added_on DATETIME
    )",
    )
    .execute(&mut db.acquire().await?)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS pending_clients (
        token TEXT UNIQUE,
        user TEXT,
        added_on DATETIME
    )",
    )
    .execute(&mut db.acquire().await?)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS documents (
        name TEXT,
        user TEXT,
        revision TEXT,
        contents TEXT,
        public BOOLEAN,
        added_on DATETIME,
        uuid TEXT UNIQUE,
        encrypted INT default 0,
        published_title TEXT,
        publish_date DATETIME
    )",
    )
    .execute(&mut db.acquire().await?)
    .await?;

    Ok(())
}
