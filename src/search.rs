use bon::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    enums::Column,
    FogBugzClient, ResponseError,
};

#[derive(Debug, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct SearchRequest {
    #[serde(rename = "q")]
    #[builder(into)]
    query: String,
    #[builder(default = vec![Column::CaseId.to_string(), Column::Title.to_string()])]
    cols: Vec<String>,
    #[serde(skip)]
    client: FogBugzClient,
}

#[derive(Debug, Deserialize)]
pub struct Event {
    #[serde(rename = "evtDescription")]
    pub description: String,
    #[serde(rename = "ixPerson")]
    pub person_id: u64,
    #[serde(rename = "sPerson")]
    pub person: String,
    #[serde(rename = "s")]
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CaseDetails {
    #[serde(rename = "ixBug")]
    pub ticket_number: u64,
    #[serde(rename = "sTitle")]
    pub title: String,
    pub events: Vec<Event>,
}

impl SearchRequest {
    pub async fn send(&self) -> Result<serde_json::Value, ResponseError> {
        let params = serde_json::json!({
            "q": self.query,
            "cols": self.cols,
        });
        self.client.send_search(params).await
    }
}

#[cfg(test)]
mod tests {
    use crate::{date::PointInTime, query::Query, FogBugzClient};

    #[tokio::test]
    async fn test_search_request() {
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

        let query = Query::builder()
            .closed_date((PointInTime::new(1, 1, 2024), PointInTime::new(31, 12, 2024)))
            .build();
        let request = api.search().query(query.to_string()).build();
        let res = request.send().await.unwrap();
        dbg!(res);
    }
}
