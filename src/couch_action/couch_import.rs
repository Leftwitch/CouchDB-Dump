use super::CouchAction;
use crate::progress_style::ProgressStyles;
use async_trait::async_trait;
use indicatif::ProgressBar;
use serde_json::{json, Value};
use std::{cmp::min, convert::TryInto, error::Error, process, sync::Arc};
use std::{fs, io::Read};
use tokio::{runtime::Handle, task::JoinHandle};
pub struct CouchImport {
    pub host: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub protocol: String,
    pub port: String,
    pub file: String,
    pub create: bool,
}

const CHUNK_SIZE: usize = 50;

#[async_trait]
impl CouchAction for CouchImport {
    async fn execute(&self) {
        println!(
            "IMPORT - HOST: {} USER: {} PW: {} FILE: {} ",
            self.host, self.user, self.password, self.file
        );

        let file_progress = ProgressBar::new(1);
        file_progress.set_style(ProgressStyles::spinner_style().clone());
        file_progress.set_prefix(&format!("[{}/{}]", 1, if self.create { 3 } else { 2 }));
        file_progress.set_message("👀 Reading input file: ");
        file_progress.enable_steady_tick(100);

        let mut file = fs::File::open(&self.file).expect("file should open read only");
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        let json: serde_json::Value =
            serde_json::from_slice(&bytes).expect("file should be proper JSON");
        let docs = json
            .get("docs")
            .expect("Docs key not provided")
            .as_array()
            .expect("Docs should not be null")
            .to_vec();

        file_progress.finish_with_message(&format!("👀 Reading input file: {} ✔️", docs.len())[..]);

        if self.create {
            if let Err(_) = self.create_db().await {
                process::exit(1);
            }
        }

        let chunks = docs.len() / CHUNK_SIZE;

        let import_progress = ProgressBar::new(docs.len().try_into().unwrap());
        import_progress.set_style(ProgressStyles::progress_style().clone());
        import_progress.set_prefix(&format!(
            "[{}/{}]",
            if self.create { 3 } else { 2 },
            if self.create { 3 } else { 2 }
        ));
        import_progress.set_message("📤 Importing documents");
        let docs_len = docs.len();
        for i in 0..(chunks + 1) {
            let lower_limit = i * CHUNK_SIZE;
            let mut upper_limit = (i + 1) * CHUNK_SIZE;
            if i == chunks {
                upper_limit = docs_len;
            }

            let protocol = self.protocol.clone();
            let host = self.host.clone();
            let port = self.port.clone();
            let db = self.database.clone();
            let user = self.user.clone();
            let pw = self.password.clone();

            let cloned = import_progress.clone();

            let docs = &docs[lower_limit..upper_limit];
            let docs = docs.to_owned();

            tokio::spawn(async move {
                let rs = CouchImport::upload_docs(&docs, protocol, host, port, db, user, pw).await;

                if rs.is_ok() {
                    cloned.inc(docs.len().try_into().unwrap());
                }
            });
        }
        import_progress.finish_with_message("📤 Importing documents ✔️ ");
    }
}

impl CouchImport {
    async fn create_db(&self) -> Result<(), Box<dyn std::error::Error>> {
        let create_progress = ProgressBar::new(1);
        create_progress.set_style(ProgressStyles::spinner_style().clone());
        create_progress.set_prefix(&format!("[{}/3]", 2));
        create_progress.set_message("✨ Creating database");
        create_progress.enable_steady_tick(100);
        let client = reqwest::Client::new();
        let url = format!(
            "{}://{}:{}/{}",
            self.protocol, self.host, self.port, self.database
        );
        let res = client
            .put(&url)
            .basic_auth(&self.user, Some(&self.password))
            .send()
            .await?;

        if !res.status().is_success() {
            println!("Error creating database: {}", res.text().await?);
            Err("Error creating database")?
        }
        create_progress.finish_with_message("✨ Creating database: ✔️");
        Ok(())
    }

    async fn upload_docs(
        docs: &[Value],
        protocol: String,
        host: String,
        port: String,
        database: String,
        user: String,
        password: String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let url = format!("{}://{}:{}/{}/_bulk_docs", protocol, host, port, database);
        let res = client
            .post(&url)
            .basic_auth(user, Some(password))
            .json(&json!({"new_edits":false, "docs": docs }))
            .send()
            .await?;

        if !res.status().is_success() {
            println!("Error uploading docs: {}", res.text().await?);
            Err("Error uploading docs")?
        }
        Ok(())
    }
}
