#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct RiteConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub app_url: String,
    pub tide_secret: String,
    pub redirect_url: String,
    pub token_url: String,
    pub file_limit: usize
}

impl RiteConfig {
    pub fn fill_from<SourceFn, ErrorType>(
        &mut self,
        f: SourceFn,
    ) -> Result<(), (&'static str, ErrorType)>
    where
        SourceFn: Fn(&'static str) -> Result<String, ErrorType>,
    {
        let set = |dest: &mut String, key: &'static str| match f(key) {
            Ok(s) => {
                *dest = s;
                Ok(())
            }
            Err(e) => Err((key, e)),
        };
        set(&mut self.app_url, "APP_URL")?;
        set(&mut self.auth_url, "AUTH_URL")?;
        set(&mut self.client_id, "CLIENT_ID")?;
        set(&mut self.client_secret, "CLIENT_SECRET")?;
        set(&mut self.redirect_url, "REDIRECT_URL")?;
        set(&mut self.tide_secret, "TIDE_SECRET")?;
        set(&mut self.token_url, "TOKEN_URL")?;
        
        let mut tmp = String::default();
        set(&mut tmp, "FILE_LIMIT")?;

        self.file_limit = tmp.parse().unwrap_or(5_000_000);

        Ok(())
    }
}
