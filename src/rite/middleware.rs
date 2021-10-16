use tide::{Middleware, Next, Request};

#[derive(Debug, Default, Clone)]
pub struct LoginMiddleware;

impl LoginMiddleware {
    /// Create a new instance of `TraceMiddleware`.
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
impl<State: Clone + Send + Sync + 'static> Middleware<State> for LoginMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        tide::log::error!("Redirecting...");
        self.check_and_redirect(req, next).await
    }
}