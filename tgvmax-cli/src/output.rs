use comfy_table::{ContentArrangement, Table};
use tgvmax_core::models::{Proposal, Station};

pub fn print_stations_table(stations: &[Station]) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Station Name"]);

    for station in stations {
        table.add_row(vec![&station.name]);
    }

    println!("{table}");
}

pub fn print_stations_json(stations: &[Station]) {
    let json = serde_json::to_string_pretty(stations).expect("failed to serialize stations");
    println!("{json}");
}

pub fn print_proposals_table(proposals: &[Proposal]) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Train #", "Departure", "Arrival", "From", "To"]);

    for p in proposals {
        table.add_row(vec![
            &p.train_number,
            &p.departure,
            &p.arrival,
            &p.origin,
            &p.destination,
        ]);
    }

    println!("{table}");
}

pub fn print_proposals_json(proposals: &[Proposal]) {
    let json = serde_json::to_string_pretty(proposals).expect("failed to serialize proposals");
    println!("{json}");
}
