use bon::Builder;
use serde::{Deserialize, Serialize};

use crate::{FogBugzClient, ResponseError};

/// Request to view hours remaining report for a milestone
#[derive(Debug, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct HoursRemainingReportRequest {
    /// Milestone ID to generate report for (required)
    #[serde(rename = "ixFixFor")]
    milestone_id: u32,
    /// API instance
    #[serde(skip)]
    client: FogBugzClient,
}

impl HoursRemainingReportRequest {
    /// Get the hours remaining report
    pub async fn send(&self) -> Result<serde_json::Value, ResponseError> {
        self.client
            .send_command("viewHoursRemainingReport", self)
            .await
    }
}

/// Request to get aggregated hours by project
#[derive(Debug, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct AggregateHoursRequest {
    /// Project ID to aggregate hours for (optional)
    #[serde(rename = "ixProject", skip_serializing_if = "Option::is_none")]
    project_id: Option<u32>,
    /// Person ID to filter by (optional)
    #[serde(rename = "ixPerson", skip_serializing_if = "Option::is_none")]
    person_id: Option<u32>,
    /// Start date for aggregation (optional)
    #[serde(rename = "dtStart", skip_serializing_if = "Option::is_none")]
    start_date: Option<String>,
    /// End date for aggregation (optional)
    #[serde(rename = "dtEnd", skip_serializing_if = "Option::is_none")]
    end_date: Option<String>,
    /// API instance
    #[serde(skip)]
    client: FogBugzClient,
}

impl AggregateHoursRequest {
    /// Get aggregated hours data
    pub async fn send(&self) -> Result<serde_json::Value, ResponseError> {
        // Use search with specific columns to get aggregated data
        let query = if let Some(project_id) = self.project_id {
            format!("project:{}", project_id)
        } else {
            "*".to_string()
        };

        let mut params = serde_json::json!({
            "q": query,
            "cols": "ixBug,sTitle,sProject,hrsElapsed,hrsCurrEst,hrsOrigEst,sPersonAssignedTo"
        });

        if let Some(person_id) = self.person_id {
            params["ixPerson"] = person_id.into();
        }
        if let Some(start_date) = &self.start_date {
            params["dtStart"] = start_date.clone().into();
        }
        if let Some(end_date) = &self.end_date {
            params["dtEnd"] = end_date.clone().into();
        }

        self.client.send_search(params).await
    }
}

/// Hours data for a case
#[derive(Debug, Deserialize, Serialize)]
pub struct CaseHours {
    #[serde(rename = "ixBug")]
    pub case_id: u32,
    #[serde(rename = "sTitle")]
    pub title: String,
    #[serde(rename = "sProject")]
    pub project: String,
    #[serde(rename = "hrsElapsed")]
    pub hours_elapsed: Option<f64>,
    #[serde(rename = "hrsCurrEst")]
    pub hours_current_estimate: Option<f64>,
    #[serde(rename = "hrsOrigEst")]
    pub hours_original_estimate: Option<f64>,
    #[serde(rename = "sPersonAssignedTo")]
    pub assigned_to: String,
}

/// Aggregated hours by project
#[derive(Debug, Serialize)]
pub struct ProjectHours {
    pub project: String,
    pub total_elapsed: f64,
    pub total_estimate: f64,
    pub case_count: u32,
}

#[cfg(test)]
mod tests {
    use crate::FogBugzClient;

    #[test]
    fn test_hours_report_builder_api() {
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

        // Test hours remaining report builder
        let _hours_report_request = api.hours_remaining_report().milestone_id(123).build();

        // Test aggregate hours builder
        let _aggregate_request = api
            .aggregate_hours()
            .project_id(456)
            .person_id(789)
            .start_date("2024-01-01".to_string())
            .end_date("2024-12-31".to_string())
            .build();

        assert!(true);
    }
}
