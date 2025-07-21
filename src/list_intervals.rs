use bon::Builder;
use chrono::NaiveDateTime;
use serde::Serialize;

use crate::{FogBugzClient, ResponseError};

#[derive(Debug, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct ListIntervalsRequest {
    #[serde(rename = "ixBug", skip_serializing_if = "Option::is_none")]
    case_id: Option<u64>,
    #[serde(rename = "ixPerson", skip_serializing_if = "Option::is_none")]
    person: Option<u64>,
    #[serde(rename = "dtStart", skip_serializing_if = "Option::is_none")]
    start_date: Option<NaiveDateTime>,
    #[serde(rename = "dtEnd", skip_serializing_if = "Option::is_none")]
    end_date: Option<NaiveDateTime>,
    #[serde(skip)]
    client: FogBugzClient,
}

impl ListIntervalsRequest {
    pub async fn send(self) -> Result<serde_json::Value, ResponseError> {
        let params = serde_json::json!({
            "ixBug": self.case_id,
            "ixPerson": self.person,
            "dtStart": self.start_date.map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string()),
            "dtEnd": self.end_date.map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string()),
        });
        self.client.send_command("listIntervals", params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_intervals_request() {
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

        let start_date =
            NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end_date =
            NaiveDateTime::parse_from_str("2024-01-31 23:59:59", "%Y-%m-%d %H:%M:%S").unwrap();

        let request = api
            .list_intervals()
            // .person(75)
            .start_date(start_date)
            .end_date(end_date)
            .build();

        let res = request.send().await;
        dbg!(&res);
        res.unwrap();
    }
}
