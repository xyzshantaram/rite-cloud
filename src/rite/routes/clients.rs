use http_types::StatusCode;
use sqlx::{Row, Sqlite};
use tera::Context;
use tide::{Redirect, Request};
use tide_tera::{context, TideTeraExt};
use uuid::Uuid;

use crate::rite::{render_error, Client, LinkRequest, State};

pub async fn link_get(req: Request<State>) -> tide::Result {
    let session = req.session();
    let mut context = Context::new();
    let mut username = String::default();
    if let Some(val) = session.get::<String>("username") {
        context.try_insert("username", &val)?;
        username = val;
    }
    let token = Uuid::new_v4();
    context.try_insert("section", "link a client")?;
    context.try_insert("token", &token.to_string())?;
    let tera = req.state().tera.clone();
    let mut db = req.state().rite_db.clone().acquire().await?;

    sqlx::query(
        "insert into pending_clients(token, user, added_on) values(?, ?, datetime('now'));",
    )
    .bind(token.to_string())
    .bind(username)
    .execute(&mut db)
    .await?;

    match tera.render_response("link_client.html", &context) {
        Ok(val) => Ok(val),
        Err(err) => {
            return render_error(
                tera,
                "Error while getting access token",
                &format!("{:?}", err),
                StatusCode::InternalServerError,
            );
        }
    }
}

pub async fn delete(req: Request<State>) -> tide::Result {
    let state = req.state();
    let tera = state.tera.clone();
    let mut db = state.rite_db.acquire().await?;
    let username: String;
    if let Some(val) = req.session().get("username") {
        username = val;
    } else {
        return render_error(
            tera,
            "Unknown error",
            "Username was None trying to read session",
            StatusCode::InternalServerError,
        );
    }

    sqlx::query("delete from clients where user = ? and uuid = ?;")
        .bind(username)
        .bind(&req.param("uuid")?)
        .execute(&mut db)
        .await?;

    Ok(Redirect::new("/clients/view").into())
}

pub async fn create(mut req: Request<State>) -> tide::Result {
    let session = req.session();
    let state = req.state();
    let db = state.rite_db.clone();
    let tera = state.tera.clone();
    let mut context = Context::new();
    let mut username = String::new();
    const EXPIRE_TIMEOUT: i32 = 300; // in seconds

    if let Some(val) = session.get::<String>("username") {
        context.try_insert("username", &val)?;
        username = val;
    }

    let link_req: LinkRequest = req.body_form().await?;

    let mut conn = db.acquire().await?;

    sqlx::query(
        "delete from pending_clients where Cast((JulianDay(datetime('now')) - JulianDay(added_on)) * 24 * 60 * 60 As Integer) > ?;"
    ).bind(EXPIRE_TIMEOUT).execute(&mut conn).await?;

    let result = sqlx::query("select * from pending_clients where token is ? and user is ?;")
        .bind(link_req.token)
        .bind(username)
        .fetch_optional(&mut conn)
        .await?;
    let token: String;
    let user: String;
    if let Some(val) = result {
        token = val.try_get("token")?;
        user = val.try_get("user")?;
    } else {
        return render_error(
            tera,
            "Invalid or expired token",
            "The token you used was either not found or expired.",
            StatusCode::Conflict,
        );
    }

    sqlx::query("delete from pending_clients where token is ? and user is ?")
        .bind(&token)
        .bind(&user)
        .execute(&mut conn)
        .await?;

    let uuid = Uuid::new_v4().to_string();
    sqlx::query(
        "insert into clients(uuid, token, user, nickname, added_on) values(?, ?, ?, ?, datetime('now'));",
    )
    .bind(uuid)
    .bind(token)
    .bind(user)
    .bind(link_req.nickname)
    .execute(&mut conn)
    .await?;

    Ok(Redirect::new("/clients/view").into())
}

pub async fn view(req: Request<State>) -> tide::Result {
    let session = req.session();
    let tera = req.state().tera.clone();
    let mut context = context! {
        "section" => "view clients"
    };
    if let Some(val) = session.get::<String>("username") {
        context.try_insert("username", &val)?;
        let username = val;
        let mut db = req.state().rite_db.clone().acquire().await?;

        let rows: Vec<Client> = sqlx::query_as::<Sqlite, Client>(
            "SELECT uuid, user, nickname, added_on from clients where user = ?;",
        )
        .bind(username)
        .fetch_all(&mut db)
        .await?;

        context.try_insert("rows", &rows)?;
        tera.render_response("view_clients.html", &context)
    } else {
        render_error(tera, "Unknown error.", "", StatusCode::InternalServerError)
    }
}
