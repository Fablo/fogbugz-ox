use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct PointInTime {
    pub day: u32,
    pub month: u32,
    pub year: u32,
}

impl PointInTime {
    pub fn new(day: u32, month: u32, year: u32) -> Self {
        Self { day, month, year }
    }
}

#[derive(Debug)]
pub struct DateRange {
    pub start: PointInTime,
    pub end: PointInTime,
}

impl DateRange {
    pub fn new(start: PointInTime, end: PointInTime) -> Self {
        Self { start, end }
    }
}

impl From<(PointInTime, PointInTime)> for DateRange {
    fn from((start, end): (PointInTime, PointInTime)) -> Self {
        Self { start, end }
    }
}

impl From<(PointInTime, PointInTime)> for Date {
    fn from((start, end): (PointInTime, PointInTime)) -> Self {
        Self::DateRange(DateRange { start, end })
    }
}

#[derive(Debug)]
pub enum Date {
    PointInTime(PointInTime),
    DateRange(DateRange),
}

impl FromStr for PointInTime {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        let day = parts[0].parse::<u32>()?;
        let month = parts[1].parse::<u32>()?;
        let year = parts[2].parse::<u32>()?;
        Ok(PointInTime { day, month, year })
    }
}

impl FromStr for DateRange {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("..").collect();
        let start = parts[0].parse::<PointInTime>()?;
        let end = parts[1].parse::<PointInTime>()?;
        Ok(DateRange { start, end })
    }
}

impl FromStr for Date {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("..") {
            Ok(Date::DateRange(s.parse::<DateRange>()?))
        } else {
            Ok(Date::PointInTime(s.parse::<PointInTime>()?))
        }
    }
}

impl fmt::Display for PointInTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}-{}-{}", self.day, self.month, self.year)
    }
}

impl fmt::Display for DateRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Date::PointInTime(point_in_time) => write!(f, "{}", point_in_time),
            Date::DateRange(date_range) => write!(f, "{}", date_range),
        }
    }
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        println!("Serializing Date: {:?}", &self);
        match self {
            Date::PointInTime(point_in_time) => {
                serializer.serialize_str(&format!("{}", point_in_time))
            }
            Date::DateRange(date_range) => serializer.serialize_str(&format!("{}", date_range)),
        }
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<Date>().map_err(serde::de::Error::custom)
    }
}

impl From<PointInTime> for Date {
    fn from(point_in_time: PointInTime) -> Self {
        Date::PointInTime(point_in_time)
    }
}

impl From<DateRange> for Date {
    fn from(date_range: DateRange) -> Self {
        Date::DateRange(date_range)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_point_in_time() {
        let point_in_time = "31-12-2020".parse::<super::PointInTime>().unwrap();
        assert_eq!(point_in_time.day, 31);
        assert_eq!(point_in_time.month, 12);
        assert_eq!(point_in_time.year, 2020);
    }

    #[test]
    fn test_parse_date_range() {
        let date_range = "01-01-2020..31-12-2020"
            .parse::<super::DateRange>()
            .unwrap();
        assert_eq!(date_range.start.day, 1);
        assert_eq!(date_range.start.month, 1);
        assert_eq!(date_range.start.year, 2020);
        assert_eq!(date_range.end.day, 31);
        assert_eq!(date_range.end.month, 12);
        assert_eq!(date_range.end.year, 2020);
    }

    #[test]
    fn test_display_point_in_time() {
        let point_in_time = super::PointInTime {
            day: 31,
            month: 12,
            year: 2020,
        };
        assert_eq!(format!("{}", point_in_time), "31-12-2020");
    }

    #[test]
    fn test_display_date_range() {
        let date_range = super::DateRange {
            start: super::PointInTime {
                day: 1,
                month: 1,
                year: 2020,
            },
            end: super::PointInTime {
                day: 31,
                month: 12,
                year: 2020,
            },
        };
        assert_eq!(format!("{}", date_range), "1-1-2020..31-12-2020");
    }
}
