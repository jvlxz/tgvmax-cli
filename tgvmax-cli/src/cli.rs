use chrono::NaiveDate;
use clap::{Parser, Subcommand};

fn parse_french_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%d/%m/%Y")
        .map_err(|_| format!("invalid date '{s}', expected DD/MM/YYYY"))
}

#[derive(Parser)]
#[command(name = "tgvmax", version, about = "Find available TGVmax 0€ trains")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output results as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Bypass station cache and fetch fresh data
    #[arg(long, global = true)]
    pub refresh: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage stations
    Station {
        #[command(subcommand)]
        action: StationAction,
    },
    /// Manage trains
    Train {
        #[command(subcommand)]
        action: TrainAction,
    },
}

#[derive(Subcommand)]
pub enum StationAction {
    /// Search for stations by name
    List {
        /// Station name to search for
        #[arg(short, long)]
        search: String,
    },
}

#[derive(Subcommand)]
pub enum TrainAction {
    /// Search for available TGVmax trains
    Search {
        /// Departure station name
        #[arg(short, long)]
        from: String,

        /// Arrival station name
        #[arg(short, long)]
        to: String,

        /// Travel date (DD/MM/YYYY). Defaults to today.
        #[arg(short, long, value_parser = parse_french_date)]
        date: Option<NaiveDate>,
    },
}
