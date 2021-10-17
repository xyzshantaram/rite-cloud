use crate::{
    rite::{server_error, Client, LinkRequest},
    State,
};
use http_types::StatusCode;
use sqlx::{Row, Sqlite, types::chrono::NaiveDateTime};
use tera::Context;
use tide::{Redirect, Request, Response};
use tide_tera::{context, TideTeraExt};
use uuid::Uuid;

use super::UploadRequest;

pub async fn homepage(req: Request<State>) -> tide::Result {
    let mut context = context! {
        "section" => "home"
    };
    let session = req.session();
    let tera = req.state().tera.clone();
    if let Some(username) = session.get::<String>("username") {
        context.try_insert("username", &username)?;
    }
    Ok(tera.render_response("index.html", &context)?.into())
}

pub async fn link_client_get(req: Request<State>) -> tide::Result {
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

    sqlx::query("insert into pending_clients(token, user, added_on) values(?, ?, datetime('now'));")
        .bind(token.to_string())
        .bind(username)
        .execute(&mut db)
        .await?;

    match tera.render_response("link_client.html", &context) {
        Ok(val) => {
            return Ok(val);
        }
        Err(err) => {
            return server_error(
                tera,
                "Error while getting access token",
                &format!("{:?}", err),
            );
        }
    }
}

pub async fn delete_client(mut req: Request<State>) -> tide::Result {
    let state = req.state();
    let tera = state.tera.clone();
    let mut db;
    if let Ok(val) = state.rite_db.acquire().await {
        db = val;
    }
    else {
        return server_error(tera, "Error acquiring database connection", "");
    }
    let username: String;
    if let Some(val) = req.session().get("username") {
        username = val;
    }
    else {
        return server_error(tera, "Unknown error", "Username was None trying to read session");
    }

    sqlx::query("delete from clients where user = ? and uuid = ?;")
    .bind(username)
    .bind(&req.param("uuid")?)
    .execute(&mut db)
    .await?;

    Ok(Redirect::new("/clients/view").into())
}

pub async fn link_client_post(mut req: Request<State>) -> tide::Result {
    let session = req.session();
    let state = req.state();
    let db = state.rite_db.clone();
    let tera = state.tera.clone();
    let mut context = Context::new();
    let mut username = String::new();
    if let Some(val) = session.get::<String>("username") {
        context.try_insert("username", &val)?;
        username = val;
    }

    let link_req: LinkRequest = req.body_form().await?;

    let mut conn = db.acquire().await?;
    let result= sqlx::query("select * from pending_clients where token is ? and user is ? and Cast((JulianDay(datetime('now')) - JulianDay(added_on)) * 24 * 60 * 60 As Integer) < ?;")
        .bind(link_req.token)
        .bind(username)
        .bind(300).fetch_optional(&mut conn).await?;

    let dt: NaiveDateTime;
    let token: String;
    let user: String;
    if let Some(val) = result {
        dt = val.try_get("added_on")?;
        token = val.try_get("token")?;
        user = val.try_get("user")?;
        println!("{}, {}, {}", dt, token, user);
    } else {
        return server_error(
            tera,
            "Invalid or expired token",
            "The token you used was either not found or expired.",
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

pub async fn view_clients(req: Request<State>) -> tide::Result {
    let session = req.session();
    let tera = req.state().tera.clone();
    let mut context = context!{
        "section" => "view clients"
    };
    let username: String;
    if let Some(val) = session.get::<String>("username") {
        username = val.clone();
        context.try_insert("username", &val)?;
    }
    else {
        return server_error(tera, "Unknown error", "Username was None trying to read session");
    }
    let mut db = req.state().rite_db.clone().acquire().await?;

    let rows: Vec<Client> = sqlx::query_as::<Sqlite, Client>("SELECT uuid, user, nickname, added_on from clients where user = ?;")
        .bind(username)
        .fetch_all(&mut db).await?;

    context.try_insert("rows", &rows)?;

    tera.render_response("view_clients.html", &context)
}

pub async fn doc_upload(mut req: Request<State>) -> tide::Result {
    let body: UploadRequest = req.body_json().await?;
    let mut res = Response::new(StatusCode::Ok);
    res.set_content_type(tide::http::mime::HTML);
    unimplemented!();
    Ok(res)
}
