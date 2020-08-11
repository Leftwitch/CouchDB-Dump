use super::CouchAction;
use crate::models::*;
use serde_json::json;
use std::{fs::File, io::Write};

pub struct CouchExport {
    pub host: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub protocol: String,
    pub file: String,
}

impl CouchAction for CouchExport {
    fn execute(&self) {
        println!(
            "EXPORT - HOST: {} USER: {} PW: {} FILE: {} ",
            self.host, self.user, self.password, self.file
        );

        let client = reqwest::Client::new();
        let url = format!(
            "{}://{}/{}/_all_docs?include_docs=true",
            self.protocol, self.host, self.database
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

        let adjusted_docs = docs
            .rows
            .iter()
            .map(|row| &row.doc)
            .collect::<Vec<&serde_json::value::Value>>();
        let final_result = json!({ "docs": adjusted_docs });
        let json = serde_json::to_string(&final_result).expect("Json Conversion Failed");

        let mut file = File::create(&self.file).expect("Output File could not be created");
        file.write_all(json.as_bytes())
            .expect("Couldnt write to Output File");

        println!("DOCS WRITTEN: {}", docs.total_rows);
    }
}
