use anyhow::Result;

use super::fetch_stations;
use crate::cli::StationAction;
use crate::output;

pub async fn run(action: StationAction, json: bool, refresh: bool) -> Result<()> {
    match action {
        StationAction::List { search } => {
            let all_stations = fetch_stations(refresh).await?;

            let search_lower = search.to_lowercase();
            let stations: Vec<_> = all_stations
                .into_iter()
                .filter(|s| s.name.to_lowercase().contains(&search_lower))
                .collect();

            if stations.is_empty() {
                anyhow::bail!("No stations found matching '{search}'");
            }

            if json {
                output::print_stations_json(&stations);
            } else {
                output::print_stations_table(&stations);
            }

            Ok(())
        }
    }
}
