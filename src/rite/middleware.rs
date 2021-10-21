use http_types::{convert::json, mime, Body, StatusCode};
use tide::{Middleware, Next, Request, Response};

use super::{State};
use crate::rite::routes::docs::UploadRequest;

#[derive(Debug, Default, Clone)]
pub struct WebAuthCheck;

impl WebAuthCheck {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    async fn check_and_redirect<'a, State: Clone + Send + Sync + 'static>(
        &'a self,
        req: Request<State>,
        next: Next<'a, State>,
    ) -> tide::Result {
        let session = req.session();
        if let None = session.get::<String>("username") {
            Ok(tide::Redirect::new("/").into())
        } else {
            Ok(next.run(req).await)
        }
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for WebAuthCheck {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        self.check_and_redirect(req, next).await
    }
}

#[derive(Debug, Default, Clone)]
pub struct ClientAuthCheck;

impl ClientAuthCheck {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    async fn check_and_respond<'a>(&'a self, mut req: Request<super::State>, next: Next<'a, super::State>) -> tide::Result {
        let body_bytes = req.body_bytes().await?;
        let body: UploadRequest = serde_json::from_slice(&body_bytes)?;

        let state = req.state();
        let mut db = state.rite_db.clone().acquire().await?;
        let mut res = Response::new(StatusCode::Ok);
        res.set_content_type(mime::JSON);

        let doc = sqlx::query("select * from clients where token is ? and user is ?;")
            .bind(body.token)
            .bind(&body.user)
            .fetch_optional(&mut db)
            .await?;

        if let None = doc {
            res.set_status(StatusCode::Unauthorized);
            res.set_body(Body::from_json(&json!({
                "message": "Invalid credentials."
            }))?);
            Ok(res)
        } else {
            req.set_body(body_bytes);
            Ok(next.run(req).await)
        }
    }
}

#[tide::utils::async_trait]
impl Middleware<State> for ClientAuthCheck {
    async fn handle(&self, req: tide::Request<State>, next: Next<'_, State>) -> tide::Result {
        self.check_and_respond(req, next).await
    }
}
