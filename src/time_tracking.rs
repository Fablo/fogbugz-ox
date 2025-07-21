use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{FogBugzClient, ResponseError};

/// Request to start working on a case (start the stopwatch)
#[derive(Debug, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct StartWorkRequest {
    /// Case ID to start working on (required)
    #[serde(rename = "ixBug")]
    case_id: u32,
    /// API instance
    #[serde(skip)]
    client: FogBugzClient,
}

impl StartWorkRequest {
    /// Start working on the case
    pub async fn send(&self) -> Result<Value, ResponseError> {
        self.client.send_command("startWork", self).await
    }
}

/// Request to stop working (stop the stopwatch)
#[derive(Debug, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct StopWorkRequest {
    /// API instance
    #[serde(skip)]
    client: FogBugzClient,
}

impl StopWorkRequest {
    /// Stop working
    pub async fn send(&self) -> Result<Value, ResponseError> {
        self.client
            .send_command("stopWork", serde_json::json!({}))
            .await
    }
}

/// Request to create a new time interval
#[derive(Debug, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct NewIntervalRequest {
    /// Case ID the interval is for (required)
    #[serde(rename = "ixBug")]
    case_id: u32,

    /// Start time of the interval (required)
    #[serde(rename = "dtStart")]
    start_time: DateTime<Utc>,

    /// End time of the interval (required)
    #[serde(rename = "dtEnd")]
    end_time: DateTime<Utc>,

    /// Description of the work done (optional)
    #[serde(rename = "sTitle", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    title: Option<String>,

    /// API instance
    #[serde(skip)]
    client: FogBugzClient,
}

impl NewIntervalRequest {
    /// Create the time interval
    pub async fn send(&self) -> Result<Value, ResponseError> {
        self.client.send_command("newInterval", self).await
    }
}

/// A time interval record
#[derive(Debug, Deserialize, Serialize)]
pub struct TimeInterval {
    #[serde(rename = "ixInterval")]
    pub id: u32,
    #[serde(rename = "ixPerson")]
    pub person_id: u32,
    #[serde(rename = "ixBug")]
    pub case_id: u32,
    #[serde(rename = "dtStart")]
    pub start_time: DateTime<Utc>,
    #[serde(rename = "dtEnd")]
    pub end_time: DateTime<Utc>,
    #[serde(rename = "sTitle")]
    pub title: String,
    #[serde(rename = "fDeleted")]
    pub is_deleted: bool,
}

impl FogBugzClient {
    /// List time intervals for a specific person and date range
    pub async fn list_time_intervals(
        &self,
        person_id: Option<u32>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<TimeInterval>, ResponseError> {
        let mut params = serde_json::json!({});
        if let Some(id) = person_id {
            params["ixPerson"] = id.into();
        }
        if let Some(start) = start_date {
            params["dtStart"] = start.format("%Y-%m-%dT%H:%M:%S").to_string().into();
        }
        if let Some(end) = end_date {
            params["dtEnd"] = end.format("%Y-%m-%dT%H:%M:%S").to_string().into();
        }

        let response = self.send_command("listIntervals", params).await?;
        let intervals = serde_json::from_value(response["data"]["intervals"].clone())?;
        Ok(intervals)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use crate::FogBugzClient;


    #[test]
    fn test_time_tracking_builder_api() {
        // Test that the builder API compiles and creates valid request structures

        #[cfg(feature = "leaky-bucket")]
        let limiter = leaky_bucket::RateLimiter::builder()
            .initial(1)
            .interval(std::time::Duration::from_secs(1))
            .build();
        #[cfg(feature = "leaky-bucket")]
        let api = FogBugzClient::builder()
            .url("https://example.com")
            .api_key("test_key")
            .limiter(limiter)
            .build();
        #[cfg(not(feature = "leaky-bucket"))]
        let api = FogBugzClient::builder()
            .url("https://example.com")
            .api_key("test_key")
            .build();

        let now = Utc::now();
        let start_time = now - Duration::hours(2);
        let end_time = now - Duration::hours(1);

        // Test start work builder
        let _start_work_request = api.start_work().case_id(123).build();

        // Test stop work builder
        let _stop_work_request = api.stop_work().build();

        // Test new interval builder
        let _new_interval_request = api
            .new_interval()
            .case_id(123)
            .start_time(start_time)
            .end_time(end_time)
            .title("Test work".to_string())
            .build();

        // All builders should compile without errors
        assert!(true);
    }

    #[tokio::test]
    async fn test_list_time_intervals() {
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

        // Test listing intervals (read-only, safe to run)
        let intervals = api.list_time_intervals(None, None, None).await.unwrap();
        println!("Found {} time intervals", intervals.len());

        // Verify data structure
        for interval in intervals.iter().take(3) {
            assert!(interval.id > 0);
            assert!(interval.case_id > 0);
            assert!(interval.person_id > 0);
            assert!(interval.start_time < interval.end_time);
        }
    }
}
