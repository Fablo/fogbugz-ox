use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

#[derive(Debug, AsRefStr, Display, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Column {
    #[strum(serialize = "ixBug", to_string = "ixBug")]
    #[strum(serialize = "caseid")]
    CaseId,
    #[strum(serialize = "sTitle", to_string = "sTitle")]
    #[strum(serialize = "title")]
    Title,
    #[strum(serialize = "sHtmlBody", to_string = "sHtmlBody")]
    #[strum(serialize = "body")]
    Body,
    #[strum(serialize = "events", to_string = "events")]
    Events,
    #[strum(serialize = "sProject", to_string = "sProject")]
    #[strum(serialize = "project")]
    Project,
    #[strum(serialize = "ixProject", to_string = "ixProject")]
    #[strum(serialize = "projectid")]
    ProjectId,
    #[strum(serialize = "sArea", to_string = "sArea")]
    #[strum(serialize = "area")]
    Area,
    #[strum(serialize = "ixPriority", to_string = "ixPriority")]
    #[strum(serialize = "priority")]
    Priority,
    #[strum(serialize = "ixStatus", to_string = "ixStatus")]
    #[strum(serialize = "status")]
    Status,
    #[strum(serialize = "ixCategory", to_string = "ixCategory")]
    #[strum(serialize = "category")]
    Category,
    #[strum(serialize = "fOpen", to_string = "fOpen")]
    #[strum(serialize = "isopen")]
    IsOpen,
    #[strum(serialize = "customFields", to_string = "customFields")]
    #[strum(serialize = "customfields")]
    CustomFields,
}

#[derive(Debug, strum::Display)]
#[repr(u8)]
pub enum Category {
    Bug = 1,
    Feature = 2,
    Inquiry = 3,
    Schedule = 4,
    Report = 5,
    Emergency = 6,
    Review = 7,
}

impl<'de> Deserialize<'de> for Category {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            1 => Ok(Category::Bug),
            2 => Ok(Category::Feature),
            3 => Ok(Category::Inquiry),
            4 => Ok(Category::Schedule),
            5 => Ok(Category::Report),
            6 => Ok(Category::Emergency),
            7 => Ok(Category::Review),
            _ => Err(serde::de::Error::custom(format!("invalid category value: {}", value))),
        }
    }
}

impl Serialize for Category {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Use the Display trait implementation (derived via strum::Display)
        // to serialize the enum variant as a string.
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, strum::Display)]
#[repr(u8)]
pub enum Priority {
    Blocker = 1,
    MuyImportante = 2,
    ShouldDo = 3,
    FixIfTime = 4,
    OhWell = 5,
    WhoCares = 6,
    DontFix = 7,
}

impl<'de> Deserialize<'de> for Priority {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            1 => Ok(Priority::Blocker),
            2 => Ok(Priority::MuyImportante),
            3 => Ok(Priority::ShouldDo),
            4 => Ok(Priority::FixIfTime),
            5 => Ok(Priority::OhWell),
            6 => Ok(Priority::WhoCares),
            7 => Ok(Priority::DontFix),
            _ => Err(serde::de::Error::custom(format!("invalid priority value: {}", value))),
        }
    }
}

impl Serialize for Priority {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Use the Display trait implementation (derived via strum::Display)
        // to serialize the enum variant as a string.
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, strum::Display)]
pub enum Status {
    Active,
    Resolved,
    Approved,
    Rejected,
    WontReview,
    AbandonedNoConsensus,
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Status, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let status = i32::deserialize(deserializer)?;
        match status {
            1 | 17 | 20 | 23 | 26 | 33 | 36 | 37 | 40 => Ok(Status::Active),
            2..=16 | 18 | 19 | 21 | 22 | 24 | 25 | 31 | 32 | 34 | 35 | 38 | 39 => {
                Ok(Status::Resolved)
            }
            27 => Ok(Status::Approved),
            28 => Ok(Status::Rejected),
            29 => Ok(Status::WontReview),
            30 => Ok(Status::AbandonedNoConsensus),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown status type: {}",
                status
            ))),
        }
    }
}

impl Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Use the Display trait implementation (derived via strum::Display)
        // to serialize the enum variant as a string.
        serializer.serialize_str(&self.to_string())
    }
}

// //       {
// //         "ixStatus": 26,
// //         "sStatus": "Active",
// //         "ixCategory": 5,
// //         "fWorkDone": false,
// //         "fResolved": false,
// //         "fDuplicate": false,
// //         "fDeleted": false,
// //         "fReactivate": false,
// //         "iOrder": 0
// //       },
// //       {
// //         "ixStatus": 27,
// //         "sStatus": "Approved",
// //         "ixCategory": 5,
// //         "fWorkDone": false,
// //         "fResolved": true,
// //         "fDuplicate": false,
// //         "fDeleted": false,
// //         "fReactivate": false,
// //         "iOrder": 1
// //       },
// //       {
// //         "ixStatus": 28,
// //         "sStatus": "Rejected",
// //         "ixCategory": 5,
// //         "fWorkDone": false,
// //         "fResolved": true,
// //         "fDuplicate": false,
// //         "fDeleted": false,
// //         "fReactivate": false,
// //         "iOrder": 2
// //       },
// //       {
// //         "ixStatus": 29,
// //         "sStatus": "Won't Review",
// //         "ixCategory": 5,
// //         "fWorkDone": false,
// //         "fResolved": true,
// //         "fDuplicate": false,
// //         "fDeleted": true,
// //         "fReactivate": false,
// //         "iOrder": 3
// //       },
// //       {
// //         "ixStatus": 30,
// //         "sStatus": "Abandoned - No Consensus",
// //         "ixCategory": 5,
// //         "fWorkDone": false,
// //         "fResolved": true,
// //         "fDuplicate": false,
// //         "fDeleted": false,
// //         "fReactivate": false,
// //         "iOrder": 3
// //       },
// //       {
// //         "ixStatus": 31,
// //         "sStatus": "Resolved (Postponed)",
// //         "ixCategory": 4,
// //         "fWorkDone": false,
// //         "fResolved": true,
// //         "fDuplicate": false,
// //         "fDeleted": false,
// //         "fReactivate": true,
// //         "iOrder": 0
// //       },
// //       {
// //         "ixStatus": 32,
// //         "sStatus": "Resolved (Postponed)",
// //         "ixCategory": 2,
// //         "fWorkDone": false,
// //         "fResolved": true,
// //         "fDuplicate": false,
// //         "fDeleted": false,
// //         "fReactivate": true,
// //         "iOrder": 0
// //       },
// //       {
// //         "ixStatus": 33,
// //         "sStatus": "Active",
// //         "ixCategory": 6,
// //         "fWorkDone": false,
// //         "fResolved": false,
// //         "fDuplicate": false,
// //         "fDeleted": false,
// //         "fReactivate": false,
// //         "iOrder": 2
// //       },
// //       {
// //         "ixStatus": 34,
// //         "sStatus": "Resolved (Completed)",
// //         "ixCategory": 6,
// //         "fWorkDone": true,
// //         "fResolved": true,
// //         "fDuplicate": false,
// //         "fDeleted": false,
// //         "fReactivate": false,
// //         "iOrder": 0
// //       },
// //       {
// //         "ixStatus": 35,
// //         "sStatus": "Resolved (Duplicate)",
// //         "ixCategory": 6,
// //         "fWorkDone": false,
// //         "fResolved": true,
// //         "fDuplicate": true,
// //         "fDeleted": false,
// //         "fReactivate": false,
// //         "iOrder": 1
// //       },
// //       {
// //         "ixStatus": 36,
// //         "sStatus": "Active new",
// //         "ixCategory": 6,
// //         "fWorkDone": false,
// //         "fResolved": false,
// //         "fDuplicate": false,
// //         "fDeleted": true,
// //         "fReactivate": false,
// //         "iOrder": 1
// //       },
// //       {
// //         "ixStatus": 37,
// //         "sStatus": "Active",
// //         "ixCategory": 7,
// //         "fWorkDone": false,
// //         "fResolved": false,
// //         "fDuplicate": false,
// //         "fDeleted": false,
// //         "fReactivate": false,
// //         "iOrder": 0
// //       },
// //       {
// //         "ixStatus": 38,
// //         "sStatus": "Resolved (Completed)",
// //         "ixCategory": 7,
// //         "fWorkDone": true,
// //         "fResolved": true,
// //         "fDuplicate": false,
// //         "fDeleted": false,
// //         "fReactivate": false,
// //         "iOrder": 0
// //       },
// //       {
// //         "ixStatus": 39,
// //         "sStatus": "Resolved (Duplicate)",
// //         "ixCategory": 7,
// //         "fWorkDone": false,
// //         "fResolved": true,
// //         "fDuplicate": true,
// //         "fDeleted": false,
// //         "fReactivate": false,
// //         "iOrder": 1
// //       },
// //       {
// //         "ixStatus": 40,
// //         "sStatus": "Active (waiting for pricing)",
// //         "ixCategory": 3,
// //         "fWorkDone": false,
// //         "fResolved": false,
// //         "fDuplicate": false,
// //         "fDeleted": false,
// //         "fReactivate": false,
// //         "iOrder": 1
// //       }
