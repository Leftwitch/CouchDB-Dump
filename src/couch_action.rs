use clap::ArgMatches;

pub trait CouchAction {
    fn execute(&self);
}

pub mod couch_export;
pub mod couch_import;
