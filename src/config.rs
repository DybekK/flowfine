pub struct SmtConfig {
    pub username: String,
    pub password: String,
    pub directory: String,
    pub keyspace: String,
    pub host: String,
    pub port: u16,
}

impl SmtConfig {
    pub fn url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
