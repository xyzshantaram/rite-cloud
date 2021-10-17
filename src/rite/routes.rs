use crate::{
    rite::{server_error, LinkRequest},
    State,
};
use http_types::StatusCode;
use sqlx::{types::chrono::NaiveDateTime, Execute, Executor, Row};
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
    let uuid = Uuid::new_v4();
    context.try_insert("section", "link a client")?;
    context.try_insert("code", &uuid.to_string())?;
    let tera = req.state().tera.clone();
    let mut db = req.state().rite_db.clone().acquire().await?;

    sqlx::query("insert into pending_clients(uuid, user, added_on) values(?, ?, datetime('now'));")
        .bind(uuid.to_string())
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
    let result= sqlx::query("select * from pending_clients where uuid is ? and user is ? and Cast((JulianDay(datetime('now')) - JulianDay(added_on)) * 24 * 60 * 60 As Integer) < ?;")
        .bind(link_req.code)
        .bind(username)
        .bind(300).fetch_optional(&mut conn).await?;

    let dt: NaiveDateTime;
    let uuid: String;
    let user: String;
    if let Some(val) = result {
        dt = val.try_get("added_on")?;
        uuid = val.try_get("uuid")?;
        user = val.try_get("user")?;
        println!("{}, {}, {}", dt, uuid, user);
    } else {
        return server_error(
            tera,
            "Invalid or expired token",
            "The token you used was either not found or expired.",
        );
    }

    sqlx::query("delete from pending_clients where uuid is ? and user is ?")
        .bind(&uuid)
        .bind(&user)
        .execute(&mut conn)
        .await?;

    sqlx::query("insert into clients(uuid, user, nickname, added_on) values(?, ?, ?, datetime('now'));")
        .bind(uuid)
        .bind(user)
        .bind(link_req.nickname)
        .execute(&mut conn)
        .await?;

    Ok(Redirect::new("/clients/view").into())
}

pub async fn view_clients(req: Request<State>) -> tide::Result {
    let session = req.session();
    let mut context = Context::new();
    if let Some(username) = session.get::<String>("username") {
        context.try_insert("username", &username)?;
    }
    unimplemented!();
}

pub async fn doc_upload(mut req: Request<State>) -> tide::Result {
    let body: UploadRequest = req.body_json().await?;
    let mut res = Response::new(StatusCode::Ok);
    res.set_content_type(tide::http::mime::HTML);
    unimplemented!();
    Ok(res)
}
