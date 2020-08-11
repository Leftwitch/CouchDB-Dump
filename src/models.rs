use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AllDocs {
    pub total_rows: i64,
    pub offset: i64,
    pub rows: Vec<Row>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Row {
    pub id: String,
    pub key: String,
    pub doc: serde_json::value::Value,
}
