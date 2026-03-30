use std::collections::HashSet;

use anyhow::Result;
use chrono::{Local, NaiveTime};
use tgvmax_core::client::TgvmaxClient;
use tgvmax_core::models::{Proposal, SearchParams, Station};
use tgvmax_core::opendata::OpenDataClient;

use super::fetch_stations;
use crate::cli::TrainAction;
use crate::output;

const MAX_STATION_MATCHES: usize = 5;

/// Deduplicate proposals across multiple origin/destination queries.
///
/// A train number uniquely identifies a service on a given date, so the same
/// physical train queried from different stations (e.g. "PARIS (intramuros)"
/// vs "PARIS MONTPARNASSE") will have different departure times but the same
/// train number. We keep only the first occurrence.
fn dedup_cross_query(batches: Vec<Vec<Proposal>>) -> Vec<Proposal> {
    let mut seen = HashSet::new();
    let mut proposals = Vec::new();
    for batch in batches {
        for p in batch {
            if seen.insert(p.train_number.clone()) {
                proposals.push(p);
            }
        }
    }
    proposals
}

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
            let mut batches = Vec::new();

            for origin in &origins {
                for destination in &destinations {
                    let params = SearchParams {
                        origin: origin.clone(),
                        destination: destination.clone(),
                        date,
                    };

                    match client.search_trains(&params).await {
                        Ok(results) => batches.push(results),
                        Err(e) => {
                            eprintln!("Warning: failed to search {origin} → {destination}: {e}");
                        }
                    }
                }
            }

            let mut proposals = dedup_cross_query(batches);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn proposal(train: &str, departure: &str, origin: &str, destination: &str) -> Proposal {
        Proposal {
            train_number: train.to_string(),
            departure: departure.to_string(),
            arrival: "10:56".to_string(),
            origin: origin.to_string(),
            destination: destination.to_string(),
        }
    }

    #[test]
    fn dedup_same_train_different_departure_times() {
        let batch1 = vec![proposal(
            "6607",
            "09:00",
            "PARIS (intramuros)",
            "LYON (intramuros)",
        )];
        let batch2 = vec![proposal(
            "6607",
            "08:55",
            "PARIS MONTPARNASSE",
            "LYON (intramuros)",
        )];

        let result = dedup_cross_query(vec![batch1, batch2]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].train_number, "6607");
    }

    #[test]
    fn dedup_keeps_different_trains() {
        let batch1 = vec![proposal(
            "6607",
            "09:00",
            "PARIS (intramuros)",
            "LYON (intramuros)",
        )];
        let batch2 = vec![proposal(
            "6609",
            "11:00",
            "PARIS (intramuros)",
            "LYON (intramuros)",
        )];

        let result = dedup_cross_query(vec![batch1, batch2]);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].train_number, "6607");
        assert_eq!(result[1].train_number, "6609");
    }
}
