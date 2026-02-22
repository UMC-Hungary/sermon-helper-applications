use anyhow::Result;
use postgresql_embedded::{PostgreSQL, Settings};
use std::path::PathBuf;

pub struct EmbeddedDb {
    pub pg: PostgreSQL,
    pub connection_url: String,
}

impl EmbeddedDb {
    pub async fn start(data_dir: PathBuf) -> Result<Self> {
        let db_name = "sermon_helper";

        let settings = Settings {
            port: 15432,
            installation_dir: data_dir.join("pg_install"),
            data_dir: data_dir.join("pg_data"),
            temporary: false,
            ..Settings::default()
        };

        let mut pg = PostgreSQL::new(settings);
        pg.setup().await?;
        pg.start().await?;

        if !pg.database_exists(db_name).await? {
            pg.create_database(db_name).await?;
        }

        let connection_url = pg.settings().url(db_name);

        Ok(Self { pg, connection_url })
    }

    pub async fn stop(self) -> Result<()> {
        self.pg.stop().await?;
        Ok(())
    }
}
