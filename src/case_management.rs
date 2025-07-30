use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{FogBugzClient, ResponseError, enums::Category};

/// Request to create a new case
#[derive(Debug, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct NewCaseRequest {
    /// Case title (required)
    #[serde(rename = "sTitle")]
    title: String,

    /// Initial case description/event (required)
    #[serde(rename = "sEvent")]
    description: String,

    /// Project ID to create the case in (optional)
    #[serde(rename = "ixProject", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    project_id: Option<u64>,

    /// Project name to create the case in (optional)
    #[serde(rename = "sProject", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    project: Option<String>,

    /// Area name within the project (optional)
    #[serde(rename = "sArea", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    area: Option<String>,

    /// Case category (optional)
    #[serde(rename = "ixCategory", skip_serializing_if = "Option::is_none")]
    category: Option<Category>,

    /// Person to assign the case to (optional)
    #[serde(rename = "ixPersonAssignedTo", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    assigned_to_id: Option<u64>,

    /// Priority level (optional)
    #[serde(rename = "ixPriority", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    priority: Option<u64>,

    /// Milestone/FixFor (optional)
    #[serde(rename = "ixFixFor", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    milestone: Option<u64>,

    /// Tags (comma-separated string, optional)
    #[serde(rename = "sTags", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    tags: Option<String>,

    /// API instance
    #[serde(skip)]
    client: FogBugzClient,
}

/// Response from creating a new case
#[derive(Debug, Serialize, Deserialize)]
pub struct NewCaseResponse {
    /// The newly created case ID
    #[serde(rename = "ixBug")]
    pub case_id: u64,
}

impl NewCaseRequest {
    /// Create a new case
    pub async fn send(&self) -> Result<NewCaseResponse, ResponseError> {
        let response = self.client.send_command("new", self).await?;

        // Extract the case ID from the response
        let case_id = response["data"]["case"]["ixBug"].as_u64().ok_or_else(|| {
            use std::io;
            ResponseError::JsonError(serde_json::Error::io(io::Error::new(
                io::ErrorKind::InvalidData,
                "Missing or invalid case ID in response",
            )))
        })? as u64;

        Ok(NewCaseResponse { case_id })
    }
}

/// Request to edit an existing case
#[derive(Debug, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct EditCaseRequest {
    /// Case ID to edit (required)
    #[serde(rename = "ixBug")]
    case_id: u64,

    /// New case title (optional)
    #[serde(rename = "sTitle", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    title: Option<String>,

    /// Event/comment to add (optional)
    #[serde(rename = "sEvent", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    event: Option<String>,

    /// Project ID to move case to (optional)
    #[serde(rename = "ixProject", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    project_id: Option<u64>,

    /// Area name within the project (optional)
    #[serde(rename = "sArea", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    area: Option<String>,

    /// Case category (optional)
    #[serde(rename = "ixCategory", skip_serializing_if = "Option::is_none")]
    category: Option<Category>,

    /// Priority level (optional)
    #[serde(rename = "ixPriority", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    priority: Option<u64>,

    /// Milestone/FixFor (optional)
    #[serde(rename = "ixFixFor", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    milestone: Option<u64>,

    /// Tags (comma-separated string, optional)
    #[serde(rename = "sTags", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    tags: Option<String>,

    /// API instance
    #[serde(skip)]
    client: FogBugzClient,
}

impl EditCaseRequest {
    /// Edit the case
    pub async fn send(&self) -> Result<Value, ResponseError> {
        self.client.send_command("edit", self).await
    }
}

/// Request to assign a case to a person
#[derive(Debug, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct AssignCaseRequest {
    /// Case ID to assign (required)
    #[serde(rename = "ixBug")]
    case_id: u64,

    /// Person ID to assign to (required)
    #[serde(rename = "ixPersonAssignedTo")]
    assigned_to_id: u64,

    /// Optional comment when assigning
    #[serde(rename = "sEvent", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    event: Option<String>,

    /// API instance
    #[serde(skip)]
    client: FogBugzClient,
}

impl AssignCaseRequest {
    /// Assign the case
    pub async fn send(&self) -> Result<Value, ResponseError> {
        self.client.send_command("assign", self).await
    }
}

/// Request to resolve a case
#[derive(Debug, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct ResolveCaseRequest {
    /// Case ID to resolve (required)
    #[serde(rename = "ixBug")]
    case_id: u64,

    /// Status to resolve to (optional, defaults to "Resolved")
    #[serde(rename = "ixStatus", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    status_id: Option<u64>,

    /// Person to assign resolved case to (optional)
    #[serde(rename = "ixPersonAssignedTo", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    assigned_to_id: Option<u64>,

    /// Resolution comment
    #[serde(rename = "sEvent", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    event: Option<String>,

    /// API instance
    #[serde(skip)]
    client: FogBugzClient,
}

impl ResolveCaseRequest {
    /// Resolve the case
    pub async fn send(&self) -> Result<Value, ResponseError> {
        self.client.send_command("resolve", self).await
    }
}

/// Request to reactivate (reopen) a resolved case
#[derive(Debug, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct ReactivateCaseRequest {
    /// Case ID to reactivate (required)
    #[serde(rename = "ixBug")]
    case_id: u64,

    /// Person to assign reactivated case to (optional)
    #[serde(rename = "ixPersonAssignedTo", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    assigned_to_id: Option<u64>,

    /// Reactivation comment
    #[serde(rename = "sEvent", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    event: Option<String>,

    /// API instance
    #[serde(skip)]
    client: FogBugzClient,
}

impl ReactivateCaseRequest {
    /// Reactivate the case
    pub async fn send(&self) -> Result<Value, ResponseError> {
        self.client.send_command("reactivate", self).await
    }
}

/// Request to close a case
#[derive(Debug, Serialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct CloseCaseRequest {
    /// Case ID to close (required)
    #[serde(rename = "ixBug")]
    case_id: u64,

    /// Closing comment
    #[serde(rename = "sEvent", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    event: Option<String>,

    /// API instance
    #[serde(skip)]
    client: FogBugzClient,
}

impl CloseCaseRequest {
    /// Close the case
    pub async fn send(&self) -> Result<Value, ResponseError> {
        self.client.send_command("close", self).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_management_builder_api() {
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

        // Test new case builder
        let _new_case_request = api
            .new_case()
            .title("Test Case".to_string())
            .description("Test description".to_string())
            .category(Category::Feature)
            .build();

        // Test edit case builder
        let _edit_request = api
            .edit_case()
            .case_id(123)
            .title("Updated title".to_string())
            .build();

        // Test assign case builder
        let _assign_request = api.assign_case().case_id(123).assigned_to_id(456).build();

        // Test resolve case builder
        let _resolve_request = api
            .resolve_case()
            .case_id(123)
            .event("Resolving case".to_string())
            .build();

        // All builders should compile without errors
        assert!(true);
    }
}
