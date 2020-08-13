use super::CouchAction;
use crate::progress_style::ProgressStyles;
use indicatif::ProgressBar;
use serde_json::{json, Value};
use std::fs;
use std::{convert::TryInto, process};
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

impl CouchAction for CouchImport {
    fn execute(&self) {
        println!(
            "IMPORT - HOST: {} USER: {} PW: {} FILE: {} ",
            self.host, self.user, self.password, self.file
        );

        let file_progress = ProgressBar::new(1);
        file_progress.set_style(ProgressStyles::spinner_style().clone());
        file_progress.set_prefix(&format!("[{}/{}]", 1, if self.create { 3 } else { 2 }));
        file_progress.set_message("👀 Reading input file: ");
        file_progress.enable_steady_tick(100);
        let file = fs::File::open(&self.file).expect("file should open read only");
        let json: serde_json::Value =
            serde_json::from_reader(file).expect("file should be proper JSON");
        let docs = json
            .get("docs")
            .expect("Docs key not provided")
            .as_array()
            .expect("Docs should not be null");

        file_progress.finish_with_message(&format!("👀 Reading input file: {} ✔️", docs.len())[..]);

        if self.create && !self.create_db() {
            process::exit(1);
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
        for i in 0..(chunks + 1) {
            let lower_limit = i * CHUNK_SIZE;
            let mut upper_limit = (i + 1) * CHUNK_SIZE;
            if i == chunks {
                upper_limit = docs.len();
            }
            let upload_docs = &docs[lower_limit..upper_limit];

            self.upload_docs(&upload_docs.to_vec());

            import_progress.inc(CHUNK_SIZE.try_into().unwrap());
        }
        import_progress.finish_with_message("📤 Importing documents ✔️ ");
    }
}

impl CouchImport {
    fn create_db(&self) -> bool {
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
        let mut res = client
            .put(&url)
            .basic_auth(&self.user, Some(&self.password))
            .send()
            .expect("The CouchDB returned an error");

        if res.status() != 201 {
            println!(
                "DB creation failed with status code: {}, response: {}",
                res.status(),
                res.text().unwrap()
            );
        }

        create_progress.finish_with_message("✨ Creating database: ✔️");
        res.status() == 201
    }

    fn upload_docs(&self, docs: &Vec<Value>) {
        let client = reqwest::Client::new();
        let url = format!(
            "{}://{}:{}/{}/_bulk_docs",
            self.protocol, self.host, self.port, self.database
        );
        let mut res = client
            .post(&url)
            .basic_auth(&self.user, Some(&self.password))
            .json(&json!({"new_edits":false, "docs": docs }))
            .send()
            .expect("The CouchDB returned an error");

        if res.status() != 201 {
            println!(
                "Request failed with status code: {}, response: {}",
                res.status(),
                res.text().unwrap()
            )
        }
    }
}
