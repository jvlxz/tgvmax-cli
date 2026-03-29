use std::collections::HashSet;

use anyhow::Result;
use chrono::{Local, NaiveTime};
use tgvmax_core::client::TgvmaxClient;
use tgvmax_core::models::{SearchParams, Station};
use tgvmax_core::opendata::OpenDataClient;

use super::fetch_stations;
use crate::cli::TrainAction;
use crate::output;

const MAX_STATION_MATCHES: usize = 5;

fn resolve_stations(all_stations: &[Station], query: &str) -> Result<Vec<String>> {
    let query_lower = query.to_lowercase();
    let matches: Vec<&Station> = all_stations
        .iter()
        .filter(|s| s.name.to_lowercase().contains(&query_lower))
        .collect();

    if matches.is_empty() {
        anyhow::bail!("No stations found matching '{query}'");
    }

    if matches.len() > MAX_STATION_MATCHES {
        anyhow::bail!(
            "Too many stations match '{query}' ({}). Please be more specific.",
            matches.len()
        );
    }

    Ok(matches.iter().map(|s| s.name.clone()).collect())
}

pub async fn run(action: TrainAction, json: bool, refresh: bool) -> Result<()> {
    match action {
        TrainAction::Search { from, to, date } => {
            let all_stations = fetch_stations(refresh).await?;

            let origins = resolve_stations(&all_stations, &from)?;
            let destinations = resolve_stations(&all_stations, &to)?;
            let date = date.unwrap_or_else(|| Local::now().date_naive());

            eprintln!("Searching from: {}", origins.join(", "));
            eprintln!("Searching to: {}", destinations.join(", "));

            let client = OpenDataClient::new()?;
            let mut proposals = Vec::new();
            // Cross-query dedup: when searching multiple origin/destination pairs,
            // the same train can appear in different queries. API-level duplicates
            // within a single response are handled in opendata.rs::dedup_proposals.
            let mut seen = HashSet::new();

            for origin in &origins {
                for destination in &destinations {
                    let params = SearchParams {
                        origin: origin.clone(),
                        destination: destination.clone(),
                        date,
                    };

                    match client.search_trains(&params).await {
                        Ok(results) => {
                            for p in results {
                                let key = (p.train_number.clone(), p.departure.clone());
                                if seen.insert(key) {
                                    proposals.push(p);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: failed to search {origin} → {destination}: {e}");
                        }
                    }
                }
            }

            proposals.sort_by(|a, b| a.departure.cmp(&b.departure));

            // Filter out trains that have already departed when searching for today
            let now = Local::now();
            if date == now.date_naive() {
                let current_time = now.time();
                proposals.retain(|p| {
                    NaiveTime::parse_from_str(&p.departure, "%H:%M")
                        .map(|dep| dep > current_time)
                        .unwrap_or(true)
                });
            }

            if proposals.is_empty() {
                anyhow::bail!("No available TGVmax trains found.");
            }

            if json {
                output::print_proposals_json(&proposals);
            } else {
                output::print_proposals_table(&proposals);
            }

            Ok(())
        }
    }
}
