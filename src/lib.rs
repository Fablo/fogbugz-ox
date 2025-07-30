pub mod api_client;
pub mod case_details;
pub mod case_management;
pub mod date;
pub mod enums;
pub mod filter;
pub mod hours_report;
pub mod list_cases;
pub mod list_intervals;
pub mod organization;
pub mod query;
pub mod search;
pub mod time_tracking;

use core::fmt;
#[cfg(feature = "leaky-bucket")]
use std::sync::Arc;

use bon::Builder;
#[cfg(feature = "leaky-bucket")]
use leaky_bucket::RateLimiter;
use thiserror::Error;

#[derive(Clone, Builder)]
pub struct FogBugzClient {
    #[builder(into)]
    pub url: String,
    #[builder(into)]
    pub api_key: String,
    #[cfg(feature = "leaky-bucket")]
    #[builder(into)]
    limiter: Option<Arc<RateLimiter>>,
    #[builder(default)]
    pub client: reqwest::Client,
}

impl fmt::Debug for FogBugzClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FogbugzClient")
            .field("url", &self.url)
            .field("api_key", &"********")
            .finish()
    }
}

#[derive(Debug, Error)]
pub enum FogbugzApiBuilderError {
    #[error("Url is not specified")]
    MissingUrl,
    #[error("Api key is not specified")]
    MissingApiKey,
    #[cfg(feature = "leaky-bucket")]
    #[error("Limiter is not specified")]
    MissingLimiter,
}

impl FogBugzClient {
    pub fn new(url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            api_key: api_key.into(),
            #[cfg(feature = "leaky-bucket")]
            limiter: None,
            client: reqwest::Client::default(),
        }
    }
    pub fn new_from_env() -> Self {
        let url = std::env::var("FOGBUGZ_URL").expect("FOGBUGZ_URL environment variable not set");
        let api_key =
            std::env::var("FOGBUGZ_API_KEY").expect("FOGBUGZ_API_KEY environment variable not set");
        Self {
            url,
            api_key,
            #[cfg(feature = "leaky-bucket")]
            limiter: None,
            client: reqwest::Client::default(),
        }
    }
    pub fn list_cases(
        &self,
    ) -> list_cases::ListCasesRequestBuilder<list_cases::list_cases_request_builder::SetClient>
    {
        list_cases::ListCasesRequest::builder().client(self.clone())
    }
    pub fn case_details(
        &self,
    ) -> case_details::CaseDetailsRequestBuilder<
        case_details::case_details_request_builder::SetClient,
    > {
        case_details::CaseDetailsRequest::builder().client(self.clone())
    }
    pub fn search(
        &self,
    ) -> search::SearchRequestBuilder<search::search_request_builder::SetClient> {
        search::SearchRequest::builder().client(self.clone())
    }

    /// Create a search request specifically for time tracking data
    pub fn search_time_tracking(&self, query: impl Into<String>) -> search::SearchRequest {
        search::SearchRequest::for_time_tracking(self, query)
    }

    /// Create a search request for elapsed hours by project
    pub fn search_project_hours(&self, project_name: impl Into<String>) -> search::SearchRequest {
        search::SearchRequest::for_project_hours(self, project_name)
    }

    /// Create a search request for elapsed hours by person
    pub fn search_person_hours(&self, person_name: impl Into<String>) -> search::SearchRequest {
        search::SearchRequest::for_person_hours(self, person_name)
    }
    pub fn list_intervals(
        &self,
    ) -> list_intervals::ListIntervalsRequestBuilder<
        list_intervals::list_intervals_request_builder::SetClient,
    > {
        list_intervals::ListIntervalsRequest::builder().client(self.clone())
    }

    // Case Management Operations
    pub fn new_case(
        &self,
    ) -> case_management::NewCaseRequestBuilder<case_management::new_case_request_builder::SetClient>
    {
        case_management::NewCaseRequest::builder().client(self.clone())
    }

    pub fn edit_case(
        &self,
    ) -> case_management::EditCaseRequestBuilder<
        case_management::edit_case_request_builder::SetClient,
    > {
        case_management::EditCaseRequest::builder().client(self.clone())
    }

    pub fn assign_case(
        &self,
    ) -> case_management::AssignCaseRequestBuilder<
        case_management::assign_case_request_builder::SetClient,
    > {
        case_management::AssignCaseRequest::builder().client(self.clone())
    }

    pub fn resolve_case(
        &self,
    ) -> case_management::ResolveCaseRequestBuilder<
        case_management::resolve_case_request_builder::SetClient,
    > {
        case_management::ResolveCaseRequest::builder().client(self.clone())
    }

    pub fn reactivate_case(
        &self,
    ) -> case_management::ReactivateCaseRequestBuilder<
        case_management::reactivate_case_request_builder::SetClient,
    > {
        case_management::ReactivateCaseRequest::builder().client(self.clone())
    }

    pub fn close_case(
        &self,
    ) -> case_management::CloseCaseRequestBuilder<
        case_management::close_case_request_builder::SetClient,
    > {
        case_management::CloseCaseRequest::builder().client(self.clone())
    }

    // Time Tracking Operations
    pub fn start_work(
        &self,
    ) -> time_tracking::StartWorkRequestBuilder<time_tracking::start_work_request_builder::SetClient>
    {
        time_tracking::StartWorkRequest::builder().client(self.clone())
    }

    pub fn stop_work(
        &self,
    ) -> time_tracking::StopWorkRequestBuilder<time_tracking::stop_work_request_builder::SetClient>
    {
        time_tracking::StopWorkRequest::builder().client(self.clone())
    }

    pub fn new_interval(
        &self,
    ) -> time_tracking::NewIntervalRequestBuilder<
        time_tracking::new_interval_request_builder::SetClient,
    > {
        time_tracking::NewIntervalRequest::builder().client(self.clone())
    }

    // Hours Reporting Operations
    pub fn hours_remaining_report(
        &self,
    ) -> hours_report::HoursRemainingReportRequestBuilder<
        hours_report::hours_remaining_report_request_builder::SetClient,
    > {
        hours_report::HoursRemainingReportRequest::builder().client(self.clone())
    }

    pub fn aggregate_hours(
        &self,
    ) -> hours_report::AggregateHoursRequestBuilder<
        hours_report::aggregate_hours_request_builder::SetClient,
    > {
        hours_report::AggregateHoursRequest::builder().client(self.clone())
    }
}

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error(transparent)]
    UrlError(#[from] url::ParseError),
    #[error("FogBugz error: {0}")]
    FogbugzError(serde_json::Value),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}
