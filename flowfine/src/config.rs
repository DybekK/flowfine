pub struct MigrationConfig {
    pub directory: String,
    pub version_formatting: VersionFormatting,
    pub keyspace: String,
}

pub enum VersionFormatting {
    Numeric,
    Datetime,
}
