use async_trait::async_trait;

#[async_trait]
pub trait CouchAction {
    async fn execute(&self);
}

pub mod couch_export;
pub mod couch_import;
