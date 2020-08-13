use super::CouchAction;
use crate::models::*;
use crate::progress_style::ProgressStyles;
use indicatif::ProgressBar;
use serde_json::{json, Value};
use std::{convert::TryInto, fs::File, io::Write};
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

impl CouchAction for CouchExport {
    fn execute(&self) {
        println!(
            "EXPORT - HOST: {} USER: {} PW: {} FILE: {} ",
            self.host, self.user, self.password, self.file
        );

        let total_docs_progress = ProgressBar::new(1);
        total_docs_progress.set_style(ProgressStyles::spinner_style().clone());
        total_docs_progress.set_prefix(&format!("[{}/3]", 1));
        total_docs_progress.set_message("📄 Fetching Total documents: ");
        total_docs_progress.enable_steady_tick(100);
        let total_docs = self.get_total_docs();

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
            let client = reqwest::Client::new();
            let url = format!(
                "{}://{}:{}/{}/_all_docs?include_docs=true&limit={}&skip={}",
                self.protocol, self.host, self.port, self.database, CHUNK_SIZE, offset
            );
            let mut res = client
                .get(&url)
                .basic_auth(&self.user, Some(&self.password))
                .send()
                .expect("The CouchDB returned an error");
            if res.status() != 200 {
                println!(
                    "Request failed with status code: {}, response: {}",
                    res.status(),
                    res.text().unwrap()
                )
            }

            let docs: AllDocs = res
                .json()
                .expect("The Response returned by the CouchDB is not valid JSON");

            let mut adjusted_docs: Vec<Value> = docs
                .rows
                .iter()
                .to_owned()
                .map(|row| row.doc.clone())
                .collect::<Vec<serde_json::value::Value>>();

            all_docs.append(&mut adjusted_docs);
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
    fn get_total_docs(&self) -> usize {
        let client = reqwest::Client::new();
        let url = format!(
            "{}://{}:{}/{}",
            self.protocol, self.host, self.port, self.database
        );
        let mut res = client
            .get(&url)
            .basic_auth(&self.user, Some(&self.password))
            .send()
            .expect("The CouchDB returned an error");

        if res.status() != 200 {
            println!(
                "Request failed with status code: {}, response: {}",
                res.status(),
                res.text().unwrap()
            )
        }
        let db: DB = res
            .json()
            .expect("The Response returned by the CouchDB is not valid JSON");

        db.doc_count
    }

    fn write_file(&self, json: String) {
        let mut file = File::create(&self.file).expect("Output File could not be created");
        file.write_all(json.as_bytes())
            .expect("Couldnt write to Output File");
    }
}
