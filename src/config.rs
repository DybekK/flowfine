pub struct SmtConfig {
    pub username: String,
    pub password: String,
    pub directory: String,
    pub version_formatting: VersionFormatting,
    pub keyspace: String,
    pub host: String,
    pub port: u16,
}

pub enum VersionFormatting {
    Numeric,
    Datetime,
}

impl SmtConfig {
    pub fn url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
