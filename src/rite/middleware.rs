use http_types::{convert::json, headers::ACCESS_CONTROL_ALLOW_ORIGIN, mime, Body, StatusCode};
use tide::{Middleware, Next, Request, Response};

use super::State;
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
        if session.get::<String>("username").is_none() {
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
pub struct DocPrelimChecks;

impl DocPrelimChecks {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    async fn check_and_respond<'a>(
        &'a self,
        mut req: Request<super::State>,
        next: Next<'a, super::State>,
    ) -> tide::Result {
        let body_bytes = req.body_bytes().await?;
        let body: UploadRequest = serde_json::from_slice(&body_bytes)?;

        let state = req.state();
        let mut db = state.rite_db.clone().acquire().await?;
        let mut res = Response::new(StatusCode::Ok);
        res.set_content_type(mime::JSON);
        res.insert_header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");

        if body_bytes.len() > req.state().cfg.file_limit {
            res.set_status(StatusCode::UnprocessableEntity);
            res.set_body(json!({
                "message": "Request too large."
            }));
            return Ok(res);
        }

        let doc = sqlx::query("select * from clients where token is ? and user is ?;")
            .bind(body.token)
            .bind(&body.user)
            .fetch_optional(&mut db)
            .await?;

        if doc.is_none() {
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
impl Middleware<State> for DocPrelimChecks {
    async fn handle(&self, req: tide::Request<State>, next: Next<'_, State>) -> tide::Result {
        self.check_and_respond(req, next).await
    }
}
