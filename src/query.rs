use core::fmt;

use serde::{Deserialize, Serialize};

use crate::date::Date;

#[derive(Debug)]
pub enum Param {
    CaseId(u64),
    AssignedTo(String),
    FromEmail(String),
    OpenedDate(Date),
    ClosedDate(Date),
}

#[derive(Debug, Deserialize)]
pub struct Query {
    pub case_id: Option<u64>,
    pub assigned_to: Option<String>,
    pub from_email: Option<String>,
    pub opened_date: Option<Date>,
    pub closed_date: Option<Date>,
}

#[derive(Debug, Default)]
pub struct QueryBuilder(pub Vec<Param>);

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if let Some(case_id) = self.case_id {
            parts.push(format!("ixBug:{}", case_id));
        }
        if let Some(assigned_to) = &self.assigned_to {
            parts.push(format!("assignedTo:{}", assigned_to));
        }
        if let Some(from_email) = &self.from_email {
            parts.push(format!("from:{}", from_email));
        }
        if let Some(opened_date) = &self.opened_date {
            parts.push(format!("opened:\"{}\"", opened_date));
        }
        if let Some(closed_date) = &self.closed_date {
            parts.push(format!("closed:\"{}\"", closed_date));
        }
        let query = parts.join(" ");
        write!(f, "{}", query)
    }
}

impl Serialize for Query {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn add_param(mut self, param: Param) -> Self {
        self.0.push(param);
        self
    }
    pub fn case_id(mut self, case_id: u64) -> Self {
        self.0.push(Param::CaseId(case_id));
        self
    }
    pub fn assigned_to(mut self, assigned_to: impl AsRef<str>) -> Self {
        self.0
            .push(Param::AssignedTo(assigned_to.as_ref().to_string()));
        self
    }
    pub fn from_email(mut self, from_email: impl AsRef<str>) -> Self {
        self.0
            .push(Param::FromEmail(from_email.as_ref().to_string()));
        self
    }
    pub fn opened_date(mut self, opened_date: impl Into<Date>) -> Self {
        self.0.push(Param::OpenedDate(opened_date.into()));
        self
    }
    pub fn closed_date(mut self, closed_date: impl Into<Date>) -> Self {
        self.0.push(Param::ClosedDate(closed_date.into()));
        self
    }
    pub fn build(self) -> Query {
        let mut query = Query {
            case_id: None,
            assigned_to: None,
            from_email: None,
            opened_date: None,
            closed_date: None,
        };
        for param in self.0 {
            match param {
                Param::CaseId(case_id) => query.case_id = Some(case_id),
                Param::AssignedTo(assigned_to) => query.assigned_to = Some(assigned_to),
                Param::FromEmail(from_email) => query.from_email = Some(from_email),
                Param::OpenedDate(opened_date) => query.opened_date = Some(opened_date),
                Param::ClosedDate(closed_date) => query.closed_date = Some(closed_date),
            }
        }
        query
    }
}

impl Query {
    pub fn builder() -> QueryBuilder {
        QueryBuilder::new()
    }
}

pub trait IntoQuery {
    fn into_query(self) -> Query;
}

impl IntoQuery for Query {
    fn into_query(self) -> Query {
        self
    }
}

impl IntoQuery for QueryBuilder {
    fn into_query(self) -> Query {
        self.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_formatting_uses_spaces_not_ampersands() {
        let query = Query::builder()
            .assigned_to("John Doe")
            .case_id(123)
            .build();

        let query_string = query.to_string();

        // Should use spaces, not ampersands to separate query parts
        assert!(query_string.contains(" "));
        assert!(!query_string.contains("&"));
        assert!(query_string.contains("ixBug:123"));
        assert!(query_string.contains("assignedTo:John Doe"));

        // Verify the correct format
        assert_eq!(query_string, "ixBug:123 assignedTo:John Doe");
    }

    #[test]
    fn test_query_with_dates() {
        use crate::date::PointInTime;

        let query = Query::builder()
            .opened_date(PointInTime::new(1, 1, 2024))
            .closed_date(PointInTime::new(31, 12, 2024))
            .build();

        let query_string = query.to_string();
        println!("Query string: '{}'", query_string);

        // Should use spaces to separate query parts
        assert!(query_string.contains(" "));
        assert!(!query_string.contains("&"));
        // Check actual format produced by PointInTime
        assert!(query_string.contains("opened:"));
        assert!(query_string.contains("closed:"));
    }
}
