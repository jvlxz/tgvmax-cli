use std::collections::HashSet;
use std::time::Duration;

use reqwest::Client;

use serde::Deserialize;

use crate::client::TgvmaxClient;
use crate::error::{Result, TgvmaxError};
use crate::models::{OpenDataRecordWrapper, OpenDataResponse, Proposal, SearchParams, Station};

const TRAIN_SEARCH_URL: &str = "https://data.sncf.com/api/records/1.0/search/";
const STATION_LIST_URL: &str =
    "https://data.sncf.com/api/explore/v2.1/catalog/datasets/tgvmax/records";

/// Client for the SNCF Open Data API (tgvmax dataset).
pub struct OpenDataClient {
    http: Client,
}

impl OpenDataClient {
    pub fn new() -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(TgvmaxError::Http)?;
        Ok(Self { http })
    }
}

/// Response from the v2 group_by aggregation endpoint.
#[derive(Debug, Deserialize)]
struct V2AggResponse {
    total_count: Option<u64>,
    results: Vec<V2AggRecord>,
}

#[derive(Debug, Deserialize)]
struct V2AggRecord {
    name: Option<String>,
}

async fn check_response(resp: reqwest::Response) -> Result<reqwest::Response> {
    if resp.status().is_success() {
        return Ok(resp);
    }
    let status = resp.status().as_u16();
    let message = resp.text().await.unwrap_or_default();
    Err(TgvmaxError::Api { status, message })
}

/// Deduplicate proposals within a single API response.
///
/// The SNCF Open Data API sometimes returns duplicate records for the same
/// train. We deduplicate by (train_no, heure_depart) here. Cross-query
/// deduplication (when searching multiple origin/destination pairs) is
/// handled separately in the CLI's train command.
fn dedup_proposals(records: Vec<OpenDataRecordWrapper>) -> Vec<Proposal> {
    let mut seen = HashSet::new();
    records
        .into_iter()
        .filter(|r| {
            let key = (
                r.fields.train_no.clone().unwrap_or_default(),
                r.fields.heure_depart.clone().unwrap_or_default(),
            );
            seen.insert(key)
        })
        .filter_map(|r| r.fields.into_proposal())
        .collect()
}

impl OpenDataClient {
    /// Fetch all unique values for a field using the v2 group_by API.
    async fn fetch_grouped_stations(&self, field: &str) -> Result<Vec<String>> {
        let select = format!("{field} as name");
        let resp = self
            .http
            .get(STATION_LIST_URL)
            .query(&[
                ("select", select.as_str()),
                ("group_by", field),
                ("limit", "500"),
            ])
            .send()
            .await?;

        let resp = check_response(resp).await?;
        let body: V2AggResponse = resp
            .json()
            .await
            .map_err(|e| TgvmaxError::Parse(e.to_string()))?;

        if let Some(total) = body.total_count {
            let returned = body.results.len() as u64;
            if returned < total {
                eprintln!(
                    "Warning: station list truncated ({returned}/{total}). Some stations may be missing."
                );
            }
        }

        Ok(body.results.into_iter().filter_map(|r| r.name).collect())
    }
}

impl TgvmaxClient for OpenDataClient {
    async fn list_stations(&self) -> Result<Vec<Station>> {
        let (origins, destinations) = tokio::try_join!(
            self.fetch_grouped_stations("origine"),
            self.fetch_grouped_stations("destination"),
        )?;

        let mut seen = HashSet::new();
        let mut stations = Vec::new();

        for name in origins.into_iter().chain(destinations) {
            if seen.insert(name.clone()) {
                stations.push(Station { name });
            }
        }

        Ok(stations)
    }

    async fn search_trains(&self, params: &SearchParams) -> Result<Vec<Proposal>> {
        let date_str = params.date.format("%Y-%m-%d").to_string();

        let resp = self
            .http
            .get(TRAIN_SEARCH_URL)
            .query(&[
                ("dataset", "tgvmax"),
                ("refine.origine", params.origin.as_str()),
                ("refine.destination", params.destination.as_str()),
                ("refine.od_happy_card", "OUI"),
                ("refine.date", date_str.as_str()),
                ("sort", "heure_depart"),
                ("rows", "100"),
            ])
            .send()
            .await?;

        let resp = check_response(resp).await?;

        let body: OpenDataResponse = resp
            .json()
            .await
            .map_err(|e| TgvmaxError::Parse(e.to_string()))?;

        let returned = body.records.len() as u64;
        if returned < body.nhits {
            eprintln!(
                "Warning: train results truncated ({returned}/{} total). Some trains may be missing.",
                body.nhits
            );
        }

        Ok(dedup_proposals(body.records))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_creation_succeeds() {
        let client = OpenDataClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn deserialize_train_search_response() {
        let json = r#"{
            "nhits": 3,
            "parameters": {},
            "records": [
                {
                    "fields": {
                        "date": "2026-04-05",
                        "origine": "PARIS (intramuros)",
                        "destination": "LYON (intramuros)",
                        "train_no": "6607",
                        "heure_depart": "09:00",
                        "heure_arrivee": "10:56",
                        "od_happy_card": "OUI"
                    }
                },
                {
                    "fields": {
                        "date": "2026-04-05",
                        "origine": "PARIS (intramuros)",
                        "destination": "LYON (intramuros)",
                        "train_no": "6607",
                        "heure_depart": "09:00",
                        "heure_arrivee": "10:56",
                        "od_happy_card": "OUI"
                    }
                },
                {
                    "fields": {
                        "date": "2026-04-05",
                        "origine": "PARIS (intramuros)",
                        "destination": "LYON (intramuros)",
                        "train_no": "6609",
                        "heure_depart": "11:00",
                        "heure_arrivee": "12:56",
                        "od_happy_card": "OUI"
                    }
                }
            ]
        }"#;

        let body: OpenDataResponse = serde_json::from_str(json).unwrap();
        assert_eq!(body.nhits, 3);
        assert_eq!(body.records.len(), 3);

        // 3 records but 2 unique trains after dedup
        let proposals = dedup_proposals(body.records);
        assert_eq!(proposals.len(), 2);
        assert_eq!(proposals[0].train_number, "6607");
        assert_eq!(proposals[1].train_number, "6609");
    }

    #[test]
    fn deserialize_v2_agg_response() {
        let json = r#"{
            "total_count": 3,
            "results": [
                {"name": null},
                {"name": "PARIS (intramuros)"},
                {"name": "LYON (intramuros)"}
            ]
        }"#;

        let resp: V2AggResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.total_count, Some(3));
        let names: Vec<String> = resp.results.into_iter().filter_map(|r| r.name).collect();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "PARIS (intramuros)");
        assert_eq!(names[1], "LYON (intramuros)");
    }

    #[test]
    fn v2_agg_response_without_total_count() {
        let json = r#"{
            "results": [
                {"name": "PARIS (intramuros)"}
            ]
        }"#;

        let resp: V2AggResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.total_count, None);
        assert_eq!(resp.results.len(), 1);
    }
}
