mod rite;

use std::time::Duration;

use async_sqlx_session::SqliteSessionStore;
use http_types::cookies::SameSite;
use rite::{
    auth::{self, logout},
    middleware::LoginMiddleware,
    oauth_config::OauthConfig,
    routes, State,
};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tera::Tera;
use tide::sessions::{SessionMiddleware};

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

    let session_db = db_connection("sqlite://./storage/sessions.db?mode=rwc".to_string()).await?;
    let mut rite_db = db_connection("sqlite://./storage/rite.db?mode=rwc".to_string()).await?;
    initialise_db(&mut rite_db).await?;

    let state = State {
        gh_client: auth::gh_oauth_client(&cfg).unwrap(),
        cfg: cfg.clone(),
        tera,
        session_db: session_db.clone(),
        rite_db: rite_db,
    };

    let mut app = tide::with_state(state.clone());

    app.with(tide::log::LogMiddleware::new());
    app.with(build_session_middleware(session_db, &cfg.tide_secret).await?);

    // tide::log::start();

    app.at("/res").serve_dir("res")?;
    app.at("/").get(routes::homepage);

    app.at("/auth/github").get(auth::gh);
    app.at("/auth/logout").get(logout);
    app.at("/auth/github/authorized").get(auth::gh_authorized);

    let clients = {
        let mut app = tide::with_state(state.clone());
        app.with(LoginMiddleware::new());

        app.at("/link").get(routes::link_client_get);
        app.at("/view").get(routes::view_clients);

        app.at("/link").post(routes::link_client_post);

        app
    };

    let docs = {
        let mut app = tide::with_state(state);
        app.with(LoginMiddleware::new());

        app.at("/docs/upload").post(routes::doc_upload);
        app
    };

    app.at("/clients").nest(clients);
    app.at("/docs").nest(docs);

    app.listen(cfg.app_url).await?;
    Ok(())
}

async fn db_connection(url: String) -> tide::Result<SqlitePool> {
    Ok(SqlitePoolOptions::new().connect(&url).await?)
}

async fn build_session_middleware(
    db: SqlitePool,
    secret: &String,
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
        "
    CREATE TABLE IF NOT EXISTS clients (
        uuid TEXT UNIQUE,
        user TEXT,
        nickname TEXT,
        added_on DATE
    )",
    )
    .execute(&mut db.acquire().await?)
    .await?;

    sqlx::query(
        "
    CREATE TABLE IF NOT EXISTS pending_clients (
        uuid TEXT UNIQUE,
        user TEXT,
        added_on DATE
    )",
    )
    .execute(&mut db.acquire().await?)
    .await?;

    sqlx::query(
        "
    CREATE TABLE IF NOT EXISTS documents (
        uuid TEXT UNIQUE,
        name TEXT
    )",
    )
    .execute(&mut db.acquire().await?)
    .await?;

    Ok(())
}
