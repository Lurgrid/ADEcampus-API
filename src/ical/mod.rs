mod error;

use chrono::{DateTime, Utc};
use error::ICalError;
use lrlex::lrlex_mod;
use lrpar::lrpar_mod;
use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, VecDeque};

lrpar_mod!("ical/ical.y");
lrlex_mod!("ical/ical.l");

#[derive(Debug, Serialize)]
pub struct Event {
    #[serde(serialize_with = "serialize_as_timestamp")]
    start: DateTime<Utc>,
    #[serde(serialize_with = "serialize_as_timestamp")]
    end: DateTime<Utc>,
    summary: String,
    location: String,
    teacher: Option<String>,
    tags: VecDeque<String>,
}

fn serialize_as_timestamp<S>(d: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_i64(d.timestamp())
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end && self.summary == other.summary
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.start.cmp(&other.start) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.end.cmp(&other.end) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.summary.cmp(&other.summary)
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ICal(BTreeMap<i64, Vec<Event>>);

#[derive(Debug, Clone, Copy)]
pub struct EventFilter<'a> {
    pub summary: Option<&'a Regex>,
    pub location: Option<&'a Regex>,
    pub teacher: Option<&'a Regex>,
    pub tags: Option<&'a Regex>,
    pub all: bool,
}

pub fn parse(cal: &str, filter: EventFilter) -> Result<ICal, ICalError> {
    let lexerdef = ical_l::lexerdef();
    let lexer = lexerdef.lexer(cal);
    let (res, _) = ical_y::parse(&lexer, filter);
    res.unwrap_or(Err(ICalError::UnableEvaluateExpression))
}
