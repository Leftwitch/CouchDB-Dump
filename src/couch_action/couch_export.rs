use super::CouchAction;
use crate::models::*;
use crate::progress_style::ProgressStyles;
use indicatif::ProgressBar;
use serde_json::{json, Value};
use std::{convert::TryInto, fs::File, io::Write, process};
use async_trait::async_trait;
const CHUNK_SIZE: usize = 250;
pub struct CouchExport {
    pub host: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub protocol: String,
    pub port: String,
    pub file: String,
}

#[async_trait]
impl CouchAction for CouchExport {
    async fn execute(&self) {
        println!(
            "EXPORT - HOST: {} USER: {} PW: {} FILE: {} ",
            self.host, self.user, self.password, self.file
        );

        let total_docs_progress = ProgressBar::new(1);
        total_docs_progress.set_style(ProgressStyles::spinner_style().clone());
        total_docs_progress.set_prefix(&format!("[{}/3]", 1));
        total_docs_progress.set_message("📄 Fetching Total documents: ");
        total_docs_progress.enable_steady_tick(100);
        let total_docs: usize;
        if let Ok(total) = self.get_total_docs().await {
            total_docs = total;
        } else {
            process::exit(1);
        }

        total_docs_progress
            .finish_with_message(&format!("📄 Fetching Total documents: {} ✔️", total_docs)[..]);

        let chunks = total_docs / CHUNK_SIZE;

        let mut all_docs: Vec<Value> = Vec::new();

        let export_progress = ProgressBar::new(total_docs.try_into().unwrap());
        export_progress.set_style(ProgressStyles::progress_style().clone());
        export_progress.set_prefix(&format!("[{}/3]", 2));
        export_progress.set_message("📥 Downloading documents");
        for chunk in 0..(chunks + 1) {
            let offset = chunk * CHUNK_SIZE;
            if let Err(_) = self.download_docs(&mut all_docs, offset).await {
                break;
            }
            export_progress.inc(CHUNK_SIZE.try_into().unwrap());
        }

        export_progress.finish_with_message("📥 Download done ✔️ ");

        let final_result = json!({ "docs": all_docs });
        let json = serde_json::to_string(&final_result).expect("Json Conversion Failed");

        let file_progress = ProgressBar::new(1);
        file_progress.set_style(ProgressStyles::spinner_style().clone());
        file_progress.set_prefix(&format!("[{}/3]", 3));
        file_progress.set_message("💾 Saving output file: ");
        file_progress.enable_steady_tick(100);
        self.write_file(json);
        file_progress.finish_with_message("💾 Saving output file: success ✔️");
    }
}

impl CouchExport {
    async fn get_total_docs(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}://{}:{}/{}",
            self.protocol, self.host, self.port, self.database
        );
        let res = client
            .get(&url)
            .basic_auth(&self.user, Some(&self.password))
            .send()
            .await?;

        let success = res.status().is_success();
        let body = res.text().await?;
        if !success {
            println!("Error accessing database: {}", body);
            Err("Error accessing database")?
        }
        let db: DB = serde_json::from_str(&body[..])?;
        Ok(db.doc_count)
    }

    async fn download_docs(&self, all_docs: &mut Vec<Value>, offset: usize) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}://{}:{}/{}/_all_docs?include_docs=true&limit={}&skip={}",
            self.protocol, self.host, self.port, self.database, CHUNK_SIZE, offset
        );
        let res = client
            .get(&url)
            .basic_auth(&self.user, Some(&self.password))
            .send()
            .await?;

        let success = res.status().is_success();
        let body = res.text().await?;
        if !success {
            println!("Error downloading docs: {}", body);
            Err("Error downloading docs")?
        }

        let docs: AllDocs = serde_json::from_str(&body[..])?;

        let mut adjusted_docs: Vec<Value> = docs
            .rows
            .iter()
            .to_owned()
            .map(|row| row.doc.clone())
            .collect::<Vec<serde_json::value::Value>>();

        all_docs.append(&mut adjusted_docs);
        Ok(())
    }

    fn write_file(&self, json: String) {
        let mut file = File::create(&self.file).expect("Output File could not be created");
        file.write_all(json.as_bytes())
            .expect("Couldnt write to Output File");
    }
}
