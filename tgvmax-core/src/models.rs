use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// A train station from the SNCF Open Data API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub name: String,
}

impl std::fmt::Display for Station {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Search parameters for finding available trains.
#[derive(Debug, Clone)]
pub struct SearchParams {
    pub origin: String,
    pub destination: String,
    pub date: NaiveDate,
}

/// A single train proposal with a TGVmax free seat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub train_number: String,
    pub departure: String,
    pub arrival: String,
    pub origin: String,
    pub destination: String,
}

/// Raw record fields from the SNCF Open Data API.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenDataRecord {
    pub date: Option<String>,
    pub origine: Option<String>,
    pub destination: Option<String>,
    pub train_no: Option<String>,
    pub heure_depart: Option<String>,
    pub heure_arrivee: Option<String>,
    pub od_happy_card: Option<String>,
}

impl OpenDataRecord {
    /// Convert this raw API record into a clean `Proposal`.
    ///
    /// Returns `None` if any required field is missing — these records
    /// are unusable for display.
    pub fn into_proposal(self) -> Option<Proposal> {
        Some(Proposal {
            train_number: self.train_no?,
            departure: self.heure_depart?,
            arrival: self.heure_arrivee?,
            origin: self.origine?,
            destination: self.destination?,
        })
    }
}

/// Wrapper for a single record in the Open Data response.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenDataRecordWrapper {
    pub fields: OpenDataRecord,
}

/// Top-level Open Data API response for train search.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenDataResponse {
    pub nhits: u64,
    pub records: Vec<OpenDataRecordWrapper>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_open_data_response() {
        let json = r#"{
            "nhits": 2,
            "parameters": {},
            "records": [
                {
                    "datasetid": "tgvmax",
                    "recordid": "abc123",
                    "fields": {
                        "date": "2026-04-03",
                        "origine": "PARIS (intramuros)",
                        "destination": "LYON (intramuros)",
                        "train_no": "6607",
                        "heure_depart": "09:00",
                        "heure_arrivee": "10:56",
                        "od_happy_card": "OUI"
                    }
                },
                {
                    "datasetid": "tgvmax",
                    "recordid": "def456",
                    "fields": {
                        "date": "2026-04-03",
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

        let resp: OpenDataResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.nhits, 2);
        assert_eq!(resp.records.len(), 2);

        let first = &resp.records[0].fields;
        assert_eq!(first.train_no.as_deref(), Some("6607"));
        assert_eq!(first.heure_depart.as_deref(), Some("09:00"));
        assert_eq!(first.origine.as_deref(), Some("PARIS (intramuros)"));
        assert_eq!(first.od_happy_card.as_deref(), Some("OUI"));
    }

    #[test]
    fn deserialize_open_data_response_missing_optional_fields() {
        let json = r#"{
            "nhits": 1,
            "records": [
                {
                    "fields": {
                        "date": "2026-04-03",
                        "origine": "PARIS (intramuros)",
                        "destination": "LYON (intramuros)"
                    }
                }
            ]
        }"#;

        let resp: OpenDataResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.records.len(), 1);
        let fields = &resp.records[0].fields;
        assert!(fields.train_no.is_none());
        assert!(fields.heure_depart.is_none());
        assert!(fields.heure_arrivee.is_none());
        assert!(fields.od_happy_card.is_none());
    }

    #[test]
    fn open_data_record_into_proposal() {
        let record = OpenDataRecord {
            date: Some("2026-04-03".to_string()),
            origine: Some("PARIS (intramuros)".to_string()),
            destination: Some("LYON (intramuros)".to_string()),
            train_no: Some("6607".to_string()),
            heure_depart: Some("09:00".to_string()),
            heure_arrivee: Some("10:56".to_string()),
            od_happy_card: Some("OUI".to_string()),
        };

        let proposal = record.into_proposal().expect("should produce a proposal");
        assert_eq!(proposal.train_number, "6607");
        assert_eq!(proposal.departure, "09:00");
        assert_eq!(proposal.arrival, "10:56");
        assert_eq!(proposal.origin, "PARIS (intramuros)");
        assert_eq!(proposal.destination, "LYON (intramuros)");
    }

    #[test]
    fn into_proposal_returns_none_on_missing_train_no() {
        let record = OpenDataRecord {
            date: Some("2026-04-03".to_string()),
            origine: Some("PARIS (intramuros)".to_string()),
            destination: Some("LYON (intramuros)".to_string()),
            train_no: None,
            heure_depart: Some("09:00".to_string()),
            heure_arrivee: Some("10:56".to_string()),
            od_happy_card: Some("OUI".to_string()),
        };
        assert!(record.into_proposal().is_none());
    }

    #[test]
    fn into_proposal_returns_none_on_missing_departure() {
        let record = OpenDataRecord {
            date: Some("2026-04-03".to_string()),
            origine: Some("PARIS (intramuros)".to_string()),
            destination: Some("LYON (intramuros)".to_string()),
            train_no: Some("6607".to_string()),
            heure_depart: None,
            heure_arrivee: Some("10:56".to_string()),
            od_happy_card: Some("OUI".to_string()),
        };
        assert!(record.into_proposal().is_none());
    }

    #[test]
    fn into_proposal_returns_none_on_missing_arrival() {
        let record = OpenDataRecord {
            date: Some("2026-04-03".to_string()),
            origine: Some("PARIS (intramuros)".to_string()),
            destination: Some("LYON (intramuros)".to_string()),
            train_no: Some("6607".to_string()),
            heure_depart: Some("09:00".to_string()),
            heure_arrivee: None,
            od_happy_card: Some("OUI".to_string()),
        };
        assert!(record.into_proposal().is_none());
    }

    #[test]
    fn station_display() {
        let s = Station {
            name: "PARIS (intramuros)".to_string(),
        };
        assert_eq!(format!("{s}"), "PARIS (intramuros)");
    }
}
