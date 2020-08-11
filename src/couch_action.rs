use clap::ArgMatches;

pub trait CouchAction {
    fn execute(matches: &ArgMatches);
}

pub mod couch_export;
pub mod couch_import;
