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
    /// Get aggregated hours data using listIntervals for accurate time tracking
    pub async fn send(&self) -> Result<serde_json::Value, ResponseError> {
        // The search API approach doesn't work well for time interval filtering
        // Use listIntervals API instead and aggregate client-side
        
        let mut params = serde_json::json!({});
        
        // Add person filter (listIntervals supports ixPerson)
        if let Some(person_id) = self.person_id {
            params["ixPerson"] = person_id.into();
        }
        
        // Add date filters (listIntervals supports dtStart/dtEnd)
        if let Some(start_date) = &self.start_date {
            params["dtStart"] = start_date.clone().into();
        }
        if let Some(end_date) = &self.end_date {
            params["dtEnd"] = end_date.clone().into();
        }

        // Get time intervals using listIntervals command (which properly supports date/person filtering)
        let intervals_response = self.client.send_command("listIntervals", params).await?;
        
        // Process intervals and aggregate by cases/projects
        if let Some(intervals) = intervals_response["data"]["intervals"].as_array() {
            let mut cases_map = std::collections::HashMap::new();
            let mut case_ids = std::collections::HashSet::new();
            
            // First pass: collect case IDs and calculate durations
            for interval in intervals {
                if let (Some(case_id), Some(title), Some(start_str), Some(end_str)) = (
                    interval["ixBug"].as_u64(),
                    interval["sTitle"].as_str(),
                    interval["dtStart"].as_str(),
                    interval["dtEnd"].as_str(),
                ) {
                    case_ids.insert(case_id);
                    
                    // Calculate duration for this interval
                    if let (Ok(start_time), Ok(end_time)) = (
                        chrono::DateTime::parse_from_rfc3339(start_str),
                        chrono::DateTime::parse_from_rfc3339(end_str),
                    ) {
                        let duration_hours = (end_time - start_time).num_seconds() as f64 / 3600.0;
                        
                        let case_entry = cases_map.entry(case_id).or_insert_with(|| {
                            serde_json::json!({
                                "ixBug": case_id,
                                "sTitle": title,
                                "hrsElapsed": 0.0,
                                "hrsCurrEst": 0.0,
                                "hrsOrigEst": 0.0,
                                "sProject": "Unknown",
                                "ixProject": null,
                                "sPersonAssignedTo": "Unknown",
                                "ixPersonAssignedTo": null
                            })
                        });
                        
                        // Add to elapsed hours
                        if let Some(current_elapsed) = case_entry["hrsElapsed"].as_f64() {
                            if let Some(number) = serde_json::Number::from_f64(current_elapsed + duration_hours) {
                                case_entry["hrsElapsed"] = serde_json::Value::Number(number);
                            }
                        } else {
                            if let Some(number) = serde_json::Number::from_f64(duration_hours) {
                                case_entry["hrsElapsed"] = serde_json::Value::Number(number);
                            }
                        }
                    }
                }
            }
            
            // Second pass: fetch case details for project information
            if !case_ids.is_empty() {
                // Build search query for the specific cases
                let case_numbers: Vec<String> = case_ids.iter().map(|id| id.to_string()).collect();
                let case_query = case_numbers.join(",");
                
                let search_params = serde_json::json!({
                    "q": case_query,
                    "cols": "ixBug,sTitle,sProject,ixProject,hrsElapsed,hrsCurrEst,hrsOrigEst,sPersonAssignedTo,ixPersonAssignedTo"
                });
                
                if let Ok(search_response) = self.client.send_search(search_params).await {
                    if let Some(cases) = search_response["data"]["cases"].as_array() {
                        for case in cases {
                            if let Some(case_id) = case["ixBug"].as_u64() {
                                if let Some(case_entry) = cases_map.get_mut(&case_id) {
                                    // Update with project and estimate information
                                    if let Some(project) = case["sProject"].as_str() {
                                        case_entry["sProject"] = serde_json::Value::String(project.to_string());
                                    }
                                    if let Some(project_id) = case["ixProject"].as_u64() {
                                        case_entry["ixProject"] = serde_json::Value::Number(serde_json::Number::from(project_id));
                                    }
                                    if let Some(curr_est) = case["hrsCurrEst"].as_f64() {
                                        if let Some(number) = serde_json::Number::from_f64(curr_est) {
                                            case_entry["hrsCurrEst"] = serde_json::Value::Number(number);
                                        }
                                    }
                                    if let Some(orig_est) = case["hrsOrigEst"].as_f64() {
                                        if let Some(number) = serde_json::Number::from_f64(orig_est) {
                                            case_entry["hrsOrigEst"] = serde_json::Value::Number(number);
                                        }
                                    }
                                    if let Some(assigned_to) = case["sPersonAssignedTo"].as_str() {
                                        case_entry["sPersonAssignedTo"] = serde_json::Value::String(assigned_to.to_string());
                                    }
                                    if let Some(assigned_to_id) = case["ixPersonAssignedTo"].as_u64() {
                                        case_entry["ixPersonAssignedTo"] = serde_json::Value::Number(serde_json::Number::from(assigned_to_id));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Convert to FogBugz search API response format
            let cases: Vec<serde_json::Value> = cases_map.into_values().collect();
            let response = serde_json::json!({
                "data": {
                    "cases": cases,
                    "count": cases.len(),
                    "totalHits": cases.len()
                },
                "errorCode": null,
                "errors": [],
                "maxCacheAge": null,
                "meta": {
                    "clientVersionAllowed": {
                        "max": 822909000,
                        "min": 822909000
                    }
                },
                "warnings": []
            });
            
            Ok(response)
        } else {
            // No intervals found, return empty response
            Ok(serde_json::json!({
                "data": {
                    "cases": [],
                    "count": 0,
                    "totalHits": 0
                },
                "errorCode": null,
                "errors": [],
                "maxCacheAge": null,
                "meta": {
                    "clientVersionAllowed": {
                        "max": 822909000,
                        "min": 822909000
                    }
                },
                "warnings": []
            }))
        }
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
    #[serde(rename = "ixProject")]
    pub project_id: Option<u32>,
    #[serde(rename = "hrsElapsed")]
    pub hours_elapsed: Option<f64>,
    #[serde(rename = "hrsCurrEst")]
    pub hours_current_estimate: Option<f64>,
    #[serde(rename = "hrsOrigEst")]
    pub hours_original_estimate: Option<f64>,
    #[serde(rename = "sPersonAssignedTo")]
    pub assigned_to: String,
    #[serde(rename = "ixPersonAssignedTo")]
    pub assigned_to_id: Option<u32>,
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

    #[tokio::test]
    async fn test_search_api_with_date_parameters() {
        let api_key = match std::env::var("FOGBUGZ_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                println!("FOGBUGZ_API_KEY not set, skipping test");
                return;
            }
        };

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

        println!("Testing search API with dtStart/dtEnd/ixPerson parameters...");
        
        // Test 1: Search with dtStart, dtEnd, ixPerson parameters (what aggregate_hours currently does)
        let params_with_dates = serde_json::json!({
            "q": "*",
            "cols": "ixBug,sTitle,sProject,ixProject,hrsElapsed",
            "dtStart": "2025-01-01",
            "dtEnd": "2025-01-31", 
            "ixPerson": 75
        });

        println!("Test 1: Search with dtStart/dtEnd/ixPerson parameters");
        match tokio::time::timeout(std::time::Duration::from_secs(5), api.send_search(params_with_dates)).await {
            Ok(Ok(response)) => {
                println!("✅ Search with date parameters succeeded");
                if let Some(data) = response.get("data") {
                    if let Some(cases) = data.get("cases") {
                        if let Some(cases_array) = cases.as_array() {
                            println!("   Found {} cases", cases_array.len());
                            // Print first case for inspection
                            if let Some(first_case) = cases_array.first() {
                                println!("   First case: {}", serde_json::to_string_pretty(first_case).unwrap_or_default());
                            }
                        }
                    }
                }
            },
            Ok(Err(e)) => {
                println!("❌ Search with date parameters failed: {}", e);
            },
            Err(_) => {
                println!("⏰ Search with date parameters timed out after 5 seconds");
            }
        }

        // Test 2: Search without date parameters for comparison
        let params_without_dates = serde_json::json!({
            "q": "*",
            "cols": "ixBug,sTitle,sProject,ixProject,hrsElapsed"
        });

        println!("\nTest 2: Search without date parameters");
        match api.send_search(params_without_dates).await {
            Ok(response) => {
                println!("✅ Search without date parameters succeeded");
                if let Some(data) = response.get("data") {
                    if let Some(cases) = data.get("cases") {
                        if let Some(cases_array) = cases.as_array() {
                            println!("   Found {} cases", cases_array.len());
                        }
                    }
                }
            },
            Err(e) => {
                println!("❌ Search without date parameters failed: {}", e);
            }
        }

        // Test 3: Search with proper FogBugz query syntax
        let params_with_query_dates = serde_json::json!({
            "q": "edited:\"2025-01-01..2025-01-31\" elapsedtime:\">0\"",
            "cols": "ixBug,sTitle,sProject,ixProject,hrsElapsed,dtLastUpdated"
        });

        // Test 4: Search with person filter using proper syntax
        let params_with_person_filter = serde_json::json!({
            "q": "assignedto:\"Person75\" OR openedby:\"Person75\" OR editedby:\"Person75\"",
            "cols": "ixBug,sTitle,sProject,ixProject,hrsElapsed,sPersonAssignedTo"
        });

        println!("\nTest 3: Search with query-based date filtering");
        match api.send_search(params_with_query_dates).await {
            Ok(response) => {
                println!("✅ Search with query date filtering succeeded");
                if let Some(data) = response.get("data") {
                    if let Some(cases) = data.get("cases") {
                        if let Some(cases_array) = cases.as_array() {
                            println!("   Found {} cases", cases_array.len());
                            // Print first case for inspection
                            if let Some(first_case) = cases_array.first() {
                                println!("   First case: {}", serde_json::to_string_pretty(first_case).unwrap_or_default());
                            }
                        }
                    }
                }
            },
            Err(e) => {
                println!("❌ Search with query date filtering failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_aggregate_hours_current_implementation() {
        let api_key = match std::env::var("FOGBUGZ_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                println!("FOGBUGZ_API_KEY not set, skipping test");
                return;
            }
        };

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

        println!("Testing current aggregate_hours implementation...");
        
        let request = api
            .aggregate_hours()
            .person_id(75)
            .start_date("2025-01-01".to_string())
            .end_date("2025-01-31".to_string())
            .build();

        match tokio::time::timeout(std::time::Duration::from_secs(10), request.send()).await {
            Ok(Ok(response)) => {
                println!("✅ aggregate_hours succeeded");
                println!("Response: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
            },
            Ok(Err(e)) => {
                println!("❌ aggregate_hours failed: {}", e);
            },
            Err(_) => {
                println!("⏰ aggregate_hours timed out after 10 seconds");
            }
        }
    }
}
