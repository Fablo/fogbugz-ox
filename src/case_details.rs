use bon::Builder;
use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    FogBugzClient, ResponseError,
    enums::{Category, Column, Priority, Status},
};

#[derive(Debug, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct CaseDetailsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(field)]
    cols: Option<Vec<String>>,
    #[serde(rename = "q")]
    case_id: u64,
    #[serde(skip)]
    client: FogBugzClient,
}

impl<S: case_details_request_builder::State> CaseDetailsRequestBuilder<S> {
    pub fn add_col(mut self, col: Column) -> Self {
        match &mut self.cols {
            Some(cols) => cols.push(col.to_string()),
            None => self.cols = Some(vec![col.to_string()]),
        }
        self
    }
    pub fn cols(mut self, cols: &[Column]) -> Self {
        self.cols = Some(cols.iter().map(|s| s.to_string()).collect());
        self
    }
    pub fn default_cols(mut self) -> Self {
        self.cols = Some(vec![
            Column::CaseId.to_string(),
            Column::Title.to_string(),
            Column::Events.to_string(),
            Column::Project.to_string(),
            Column::Area.to_string(),
            Column::Priority.to_string(),
            Column::Status.to_string(),
            Column::Category.to_string(),
            Column::IsOpen.to_string(),
        ]);
        self
    }
}

#[derive(Debug, Error)]
pub enum CaseDetailsRequestBuilderError {
    #[error("Ticket number is not specified")]
    TicketNumberNotSpecified,
    #[error("Api is not specified")]
    ApiNotSpecified,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attachment {
    #[serde(rename = "sFileName")]
    pub file_name: String,
    #[serde(rename = "sURL")]
    pub url: String,
}

#[derive(Debug, strum::Display)]
pub enum EventType {
    Opened = 1,
    Edited = 2,
    Assigned = 3,
    Reactivated = 4,
    Reopened = 5,
    Closed = 6,
    Moved = 7,
    Unknown = 8,
    Replied = 9,
    Forwarded = 10,
    Received = 11,
    Sorted = 12,
    NotSorted = 13,
    Resolved = 14,
    Emailed = 15,
    ReleaseNoted = 16,
    DeletedAttachment = 17,
}

impl Serialize for EventType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Use the Display trait implementation (derived via strum::Display)
        // to serialize the enum variant as a string.
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for EventType {
    fn deserialize<D>(deserializer: D) -> Result<EventType, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let event_type = i32::deserialize(deserializer)?;
        match event_type {
            1 => Ok(EventType::Opened),
            2 => Ok(EventType::Edited),
            3 => Ok(EventType::Assigned),
            4 => Ok(EventType::Reactivated),
            5 => Ok(EventType::Reopened),
            6 => Ok(EventType::Closed),
            7 => Ok(EventType::Moved),
            8 => Ok(EventType::Unknown),
            9 => Ok(EventType::Replied),
            10 => Ok(EventType::Forwarded),
            11 => Ok(EventType::Received),
            12 => Ok(EventType::Sorted),
            13 => Ok(EventType::NotSorted),
            14 => Ok(EventType::Resolved),
            15 => Ok(EventType::Emailed),
            16 => Ok(EventType::ReleaseNoted),
            17 => Ok(EventType::DeletedAttachment),
            _ => Ok(EventType::Unknown),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "evt")]
    pub event_type: EventType,
    #[serde(rename = "evtDescription")]
    pub description: String,
    #[serde(rename = "dt")]
    pub datetime: DateTime<Utc>,
    #[serde(rename = "ixPerson")]
    pub person_id: u64,
    #[serde(rename = "sPerson")]
    pub person: String,
    #[serde(rename = "ixPersonAssignedTo")]
    pub assigned_to_id: Option<u64>,
    pub attachments: Option<Vec<Attachment>>,
    #[serde(rename = "s")]
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CaseDetails {
    #[serde(rename = "ixBug")]
    pub case_id: u64,
    #[serde(rename = "sTitle")]
    pub title: String,
    #[serde(rename = "sProject")]
    pub project: String,
    #[serde(rename = "fOpen")]
    pub is_open: bool,
    #[serde(rename = "sArea")]
    pub area: String,
    #[serde(rename = "ixStatus")]
    pub status: Status,
    #[serde(rename = "ixPriority")]
    pub priority: Priority,
    #[serde(rename = "ixCategory")]
    pub category: Category,
    pub events: Vec<Event>,
    #[serde(rename = "customFields", skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<Vec<String>>,
}

impl CaseDetailsRequest {
    pub async fn send(&self) -> Result<CaseDetails, ResponseError> {
        let url = Url::parse(&self.client.url)?.join("api/search")?;
        #[cfg(feature = "leaky-bucket")]
        if let Some(ref limiter) = self.client.limiter {
            limiter.acquire_one().await;
        }
        let mut body = serde_json::to_value(self)?;
        body["token"] = self.client.api_key.clone().into();
        let response = self
            .client
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .bearer_auth(&self.client.api_key)
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            let mut json: serde_json::Value = response.json().await?;
            if let serde_json::Value::Array(events) = &mut json["data"]["cases"][0]["events"] {
                events.retain(|event| matches!(event, serde_json::Value::Object(_)));
            }
            let case_details =
                serde_json::from_value::<CaseDetails>(json["data"]["cases"][0].take())?;
            Ok(case_details)
        } else {
            let json: serde_json::Value = response.json().await?;
            Err(ResponseError::FogbugzError(json))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FogBugzClient;

    #[tokio::test]
    async fn test_case_details_request() {
        let api_key = std::env::var("FOGBUGZ_API_KEY").unwrap();
        #[cfg(feature = "leaky-bucket")]
        let limiter = leaky_bucket::RateLimiter::builder()
            .initial(1)
            .interval(std::time::Duration::from_secs(1))
            .build();
        #[cfg(feature = "leaky-bucket")]
        let api = FogBugzClient::builder()
            .url("https://retailic.fogbugz.com")
            .api_key(api_key)
            .limiter(limiter)
            .build();
        #[cfg(not(feature = "leaky-bucket"))]
        let api = FogBugzClient::builder()
            .url("https://retailic.fogbugz.com")
            .api_key(api_key)
            .build();
        let request = api.case_details().case_id(61331).default_cols().build();
        let res = request.send().await.unwrap();
        dbg!(res);
    }
}
