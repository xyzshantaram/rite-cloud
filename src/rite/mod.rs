use oauth2::{basic::BasicClient};

pub mod oauth_config;
pub mod auth;

use oauth_config::OauthConfig;

#[derive(Clone, Debug)]
pub struct State {
    pub gh_client: BasicClient,
    pub cfg: OauthConfig,
    pub tera: tera::Tera
}