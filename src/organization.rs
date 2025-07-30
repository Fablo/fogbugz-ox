use serde::{Deserialize, Serialize};

use crate::{FogBugzClient, ResponseError};

/// A FogBugz project
#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
    #[serde(rename = "ixProject")]
    pub id: u32,
    #[serde(rename = "sProject")]
    pub name: String,
    #[serde(rename = "ixPersonOwner")]
    pub owner_id: u32,
    #[serde(rename = "sPersonOwner")]
    pub owner: String,
    #[serde(rename = "sEmail")]
    pub email: String,
    #[serde(rename = "sPhone")]
    pub phone: String,
    #[serde(rename = "fInbox")]
    pub is_inbox: bool,
    #[serde(rename = "ixWorkflow")]
    pub workflow_id: u32,
    #[serde(rename = "fDeleted")]
    pub is_deleted: bool,
}

/// A FogBugz user/person
#[derive(Debug, Deserialize, Serialize)]
pub struct Person {
    #[serde(rename = "ixPerson")]
    pub id: u32,
    #[serde(rename = "sFullName")]
    pub full_name: String,
    #[serde(rename = "sEmail")]
    pub email: String,
    #[serde(rename = "sPhone")]
    pub phone: String,
    #[serde(rename = "fAdministrator")]
    pub is_administrator: bool,
    #[serde(rename = "fCommunity")]
    pub is_community: bool,
    #[serde(rename = "fVirtual")]
    pub is_virtual: bool,
    #[serde(rename = "fDeleted")]
    pub is_deleted: bool,
    #[serde(rename = "fNotify")]
    pub notifications_enabled: bool,
    #[serde(rename = "sHomepage")]
    pub homepage: String,
    #[serde(rename = "sLocale")]
    pub locale: String,
    #[serde(rename = "sLanguage")]
    pub language: String,
    #[serde(rename = "sTimeZoneKey")]
    pub timezone: String,
}

/// A FogBugz area within a project
#[derive(Debug, Deserialize, Serialize)]
pub struct Area {
    #[serde(rename = "ixArea")]
    pub id: u32,
    #[serde(rename = "sArea")]
    pub name: String,
    #[serde(rename = "ixProject")]
    pub project_id: u32,
    #[serde(rename = "ixPersonOwner")]
    pub owner_id: u32,
    #[serde(rename = "sPersonOwner")]
    pub owner: String,
    #[serde(rename = "nType")]
    pub area_type: u32,
}

/// A FogBugz category
#[derive(Debug, Deserialize, Serialize)]
pub struct CategoryInfo {
    #[serde(rename = "ixCategory")]
    pub id: u32,
    #[serde(rename = "sCategory")]
    pub name: String,
    #[serde(rename = "sPlural")]
    pub plural: String,
    #[serde(rename = "ixStatusDefault")]
    pub default_status_id: u32,
    #[serde(rename = "fIsScheduleItem")]
    pub is_schedule_item: bool,
}

/// A FogBugz priority level
#[derive(Debug, Deserialize, Serialize)]
pub struct Priority {
    #[serde(rename = "ixPriority")]
    pub id: u32,
    #[serde(rename = "sPriority")]
    pub name: String,
}

/// A FogBugz status
#[derive(Debug, Deserialize, Serialize)]
pub struct Status {
    #[serde(rename = "ixStatus")]
    pub id: u32,
    #[serde(rename = "sStatus")]
    pub name: String,
    #[serde(rename = "ixCategory")]
    pub category_id: u32,
    #[serde(rename = "fResolved")]
    pub is_resolved: bool,
    #[serde(rename = "fDuplicate")]
    pub is_duplicate: bool,
    #[serde(rename = "fDeleted")]
    pub is_deleted: bool,
    #[serde(rename = "iOrder")]
    pub order: u32,
}

/// A FogBugz milestone/FixFor
#[derive(Debug, Deserialize, Serialize)]
pub struct Milestone {
    #[serde(rename = "ixFixFor")]
    pub id: u32,
    #[serde(rename = "sFixFor")]
    pub name: String,
    #[serde(rename = "ixProject")]
    pub project_id: u32,
    #[serde(rename = "fDeleted")]
    pub is_deleted: bool,
    #[serde(rename = "dt")]
    pub date: Option<String>,
    #[serde(rename = "dtStart")]
    pub start_date: Option<String>,
    #[serde(rename = "sStartNote")]
    pub start_note: String,
}

/// A saved filter
#[derive(Debug, Deserialize, Serialize)]
pub struct Filter {
    #[serde(rename = "sFilter")]
    pub id: String,
    #[serde(rename = "type")]
    pub filter_type: String,
    #[serde(rename = "#text", default)]
    pub name: Option<String>,
    #[serde(rename = "#cdata-section", default)]
    pub description: Option<String>,
}

impl FogBugzClient {
    /// List all projects
    pub async fn list_projects(&self) -> Result<Vec<Project>, ResponseError> {
        let response = self
            .send_command("listProjects", serde_json::json!({}))
            .await?;
        let projects = serde_json::from_value(response["data"]["projects"].clone())?;
        Ok(projects)
    }

    /// List all people/users
    pub async fn list_people(&self) -> Result<Vec<Person>, ResponseError> {
        let params = serde_json::json!({
            "fIncludeNormal": true,
            "fIncludeCommunity": true,
            "fIncludeVirtual": false
        });
        let response = self.send_command("listPeople", params).await?;
        let people = serde_json::from_value(response["data"]["people"].clone())?;
        Ok(people)
    }

    /// List areas for a specific project
    pub async fn list_areas(&self, project_id: Option<u32>) -> Result<Vec<Area>, ResponseError> {
        let mut params = serde_json::json!({});
        if let Some(id) = project_id {
            params["ixProject"] = id.into();
        }
        let response = self.send_command("listAreas", params).await?;
        let areas = serde_json::from_value(response["data"]["areas"].clone())?;
        Ok(areas)
    }

    /// List all categories
    pub async fn list_categories(&self) -> Result<Vec<CategoryInfo>, ResponseError> {
        let response = self
            .send_command("listCategories", serde_json::json!({}))
            .await?;
        let categories = serde_json::from_value(response["data"]["categories"].clone())?;
        Ok(categories)
    }

    /// List all priorities
    pub async fn list_priorities(&self) -> Result<Vec<Priority>, ResponseError> {
        let response = self
            .send_command("listPriorities", serde_json::json!({}))
            .await?;
        let priorities = serde_json::from_value(response["data"]["priorities"].clone())?;
        Ok(priorities)
    }

    /// List all statuses for a specific category
    pub async fn list_statuses(
        &self,
        category_id: Option<u32>,
    ) -> Result<Vec<Status>, ResponseError> {
        let mut params = serde_json::json!({});
        if let Some(id) = category_id {
            params["ixCategory"] = id.into();
        }
        let response = self.send_command("listStatuses", params).await?;
        let statuses = serde_json::from_value(response["data"]["statuses"].clone())?;
        Ok(statuses)
    }

    /// List milestones/FixFors for a specific project
    pub async fn list_milestones(
        &self,
        project_id: Option<u32>,
    ) -> Result<Vec<Milestone>, ResponseError> {
        let mut params = serde_json::json!({});
        if let Some(id) = project_id {
            params["ixProject"] = id.into();
        }
        let response = self.send_command("listFixFors", params).await?;
        let milestones = serde_json::from_value(response["data"]["fixfors"].clone())?;
        Ok(milestones)
    }

    /// List all saved filters
    pub async fn list_filters(&self) -> Result<Vec<Filter>, ResponseError> {
        let response = self.send_list_filters().await?;

        // Handle the complex filter structure
        let mut filters = Vec::new();
        if let Some(filter_data) = response["data"].as_object() {
            // Handle the default filter
            if let Some(default_filter) = filter_data.get("sFilter") {
                filters.push(Filter {
                    id: default_filter.as_str().unwrap_or("").to_string(),
                    filter_type: "default".to_string(),
                    name: Some("Default".to_string()),
                    description: None,
                });
            }

            // Handle the filters array
            if let Some(filters_array) = filter_data.get("filters") {
                if let Some(array) = filters_array.as_array() {
                    for filter_item in array {
                        if let Some(filter_str) = filter_item.as_str() {
                            // Simple string filter
                            filters.push(Filter {
                                id: filter_str.to_string(),
                                filter_type: "builtin".to_string(),
                                name: Some(filter_str.to_string()),
                                description: None,
                            });
                        } else if let Some(filter_obj) = filter_item.as_object() {
                            // Complex filter object
                            let id = filter_obj
                                .get("sFilter")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let filter_type = filter_obj
                                .get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let name = filter_obj
                                .get("#text")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let description = filter_obj
                                .get("#cdata-section")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            filters.push(Filter {
                                id,
                                filter_type,
                                name,
                                description,
                            });
                        }
                    }
                }
            }
        }

        Ok(filters)
    }
}

#[cfg(test)]
mod tests {
    use crate::FogBugzClient;

    #[tokio::test]
    async fn test_list_projects() {
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

        let projects = api.list_projects().await.unwrap();
        assert!(!projects.is_empty());

        for project in &projects {
            assert!(project.id > 0);
            assert!(!project.name.is_empty());
        }

        println!("Found {} projects", projects.len());
    }

    #[tokio::test]
    async fn test_list_people() {
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

        let people = api.list_people().await.unwrap();
        assert!(!people.is_empty());

        for person in &people {
            assert!(person.id > 0);
            assert!(!person.full_name.is_empty());
        }

        println!("Found {} people", people.len());
    }

    #[tokio::test]
    async fn test_list_filters() {
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

        let filters = api.list_filters().await.unwrap();
        assert!(!filters.is_empty());

        for filter in &filters {
            assert!(!filter.id.is_empty());
        }

        println!("Found {} filters", filters.len());
    }
}
