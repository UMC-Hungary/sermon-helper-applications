use anyhow::Result;
use postgresql_embedded::{PostgreSQL, Settings};
use std::path::PathBuf;
use std::time::Duration;

const PG_PORT: u16 = 15432;
const PG_USER: &str = "postgres";
const PG_PASS: &str = "sermon_helper_embedded";

pub struct EmbeddedDb {
    pub pg: PostgreSQL,
    pub connection_url: String,
}

impl EmbeddedDb {
    pub async fn start(data_dir: PathBuf) -> Result<Self> {
        let db_name = "sermon_helper";

        let settings = Settings {
            port: PG_PORT,
            installation_dir: data_dir.join("pg_install"),
            data_dir: data_dir.join("pg_data"),
            temporary: false,
            username: PG_USER.to_string(),
            password: PG_PASS.to_string(),
            ..Settings::default()
        };

        let mut pg = PostgreSQL::new(settings);
        pg.setup().await?;

        // If start fails (stale process from a previous session left on the
        // port), kill whatever is using the port and retry once.
        if let Err(e) = pg.start().await {
            tracing::warn!("PG start failed ({e}); killing stale process on port {PG_PORT} and retrying");
            kill_on_port(PG_PORT).await;
            tokio::time::sleep(Duration::from_secs(2)).await;
            pg.start().await?;
        }

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

/// Kill any processes listening on `port` using `lsof` + SIGTERM.
async fn kill_on_port(port: u16) {
    let output = match tokio::process::Command::new("lsof")
        .args(["-t", "-i", &format!("TCP:{port}")])
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => {
            tracing::warn!("lsof failed: {e}");
            return;
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    for pid_str in stdout.split_whitespace() {
        if let Ok(pid) = pid_str.parse::<u32>() {
            tracing::info!("Sending SIGTERM to stale PG process {pid}");
            let _ = tokio::process::Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .status()
                .await;
        }
    }
}
