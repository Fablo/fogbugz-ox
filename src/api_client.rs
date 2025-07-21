use reqwest::Url;
use serde::Serialize;
use serde_json::Value;

use crate::{FogBugzClient, ResponseError};

impl FogBugzClient {
    /// Send a command to the FogBugz JSON API
    pub(crate) async fn send_command<T: Serialize>(
        &self,
        cmd: &str,
        params: T,
    ) -> Result<Value, ResponseError> {
        let url = Url::parse(&self.url)?.join("f/api/0/jsonapi")?;

        #[cfg(feature = "leaky-bucket")]
        if let Some(ref limiter) = self.limiter {
            limiter.acquire_one().await;
        }

        // Build the request payload
        let mut payload = serde_json::to_value(params)?;
        payload["cmd"] = cmd.into();
        payload["token"] = self.api_key.clone().into();

        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let json: Value = response.json().await?;

            // Check for API errors in response
            if let Some(errors) = json.get("errors") {
                if let Some(errors_array) = errors.as_array() {
                    if !errors_array.is_empty() {
                        return Err(ResponseError::FogbugzError(json));
                    }
                }
            }

            Ok(json)
        } else {
            let json: Value = response.json().await?;
            Err(ResponseError::FogbugzError(json))
        }
    }


    /// Send a search command (internal API method)
    pub(crate) async fn send_search<T: Serialize>(&self, params: T) -> Result<Value, ResponseError> {
        self.send_command("search", params).await
    }

    /// Send a listCases command (internal API method)
    pub(crate) async fn send_list_cases<T: Serialize>(&self, params: T) -> Result<Value, ResponseError> {
        self.send_command("listCases", params).await
    }

    /// Send a listFilters command (internal API method)
    pub(crate) async fn send_list_filters(&self) -> Result<Value, ResponseError> {
        self.send_command("listFilters", serde_json::json!({}))
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::FogBugzClient;

    #[tokio::test]
    async fn test_api_client_search() {
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

        let client = &api;

        // Test search command
        let params = serde_json::json!({
            "q": "status:Active",
            "cols": ["sTitle", "sStatus"],
            "max": 2
        });

        let result = client.send_search(params).await.unwrap();
        assert!(result["data"]["count"].as_u64().unwrap() > 0);
        assert!(result["data"]["cases"].is_array());

        // Test listCases command
        let params = serde_json::json!({
            "sFilter": "",
            "cols": ["sTitle", "ixBug"],
            "max": 2
        });

        let result = client.send_list_cases(params).await.unwrap();
        assert!(result["data"]["count"].as_u64().unwrap() > 0);
        assert!(result["data"]["cases"].is_array());
    }
}
