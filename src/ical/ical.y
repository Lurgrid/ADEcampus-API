%start Cal
%avoid_insert "BEGIN"
%avoid_insert "END"

%avoid_insert "VCALENDAR"

%avoid_insert "METHOD"
%avoid_insert "REQUEST"
%avoid_insert "PRODID"
%avoid_insert "VERSION"
%avoid_insert "CALSCALE"
%avoid_insert "GREGORIAN"

%avoid_insert "VEVENT"

%avoid_insert "DTSTAMP"
%avoid_insert "DTSTART"
%avoid_insert "DTEND"
%avoid_insert "SUMMARY"
%avoid_insert "LOCATION"
%avoid_insert "DESCRIPTION"
%avoid_insert "UID"
%avoid_insert "CREATED"
%avoid_insert "LAST-MODIFIED"
%avoid_insert "SEQUENCE"

%avoid_insert "NUM"
%avoid_insert "DATE"

%avoid_insert "STRING"

%parse-param filter: EventFilter
%%
Cal -> Result<ICal>:
    'BEGIN' 'VCALENDAR'
    'METHOD' 'REQUEST'
    'PRODID' 'STRING'
    'VERSION' 'FLOAT'
    'CALSCALE' 'GREGORIAN' 
    LEvent
    'END' 'VCALENDAR' {
        let mut map = $11?;
        map.values_mut().for_each(|vec| vec.sort());
        Ok(ICal(map))
    }
    ;

LEvent -> Result<BTreeMap<i64, Vec<Event>>>:
    %empty {
        Ok(BTreeMap::default())
    }
    | Event LEvent {
        let mut map = $2?;
        let event = $1?;
        let mut is_filtered = false;

        let ff = [
            {
                if let Some(reg) = filter.summary {
                    is_filtered = true;
                    reg.is_match(&event.summary)
                } else {
                    filter.all
                }
            },
            {
                if let Some(reg) = filter.location {
                    is_filtered = true;
                    reg.is_match(&event.location)
                } else {
                    filter.all
                }
            },
            {
                if let Some(reg) = filter.teacher {
                    is_filtered = true;
                    if let Some(ref teacher) = event.teacher {
                        reg.is_match(teacher)
                    } else {
                        filter.all
                    }
                } else {
                    filter.all
                }           
            },
            {
                if let Some(reg) = filter.tags {
                    is_filtered = true;
                    event.tags.iter().any(|tag| reg.is_match(tag))
                } else {
                    filter.all
                }
            }
        ];
        if is_filtered {
            let result = if filter.all {
                ff.iter().all(|&x| x)
            } else {
                ff.iter().any(|&x| x)
            };
            if !result {
                return Ok(map);
            }
        }

        let key = unsafe { event
            .start
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap_unchecked()
            .and_utc()
            .timestamp()
        };
        match map.get_mut(&key) {
            Some(set) => {
                set.push(event);
            },
            None => {
                let mut set = Vec::default();
                set.push(event);
                map.insert(key, set);
            }
        }
        Ok(map)
    }
    ;

Event -> Result<Event>:
    'BEGIN' 'VEVENT'
    'DTSTAMP' 'DATE'
    'DTSTART' 'DATE'
    'DTEND' 'DATE'
    'SUMMARY' 'STRING'
    'LOCATION' 'STRING'
    'DESCRIPTION' 'STRING'
    'UID' 'STRING'
    'CREATED' 'DATE'
    'LAST-MODIFIED' 'DATE'
    'SEQUENCE' 'NUM'
    'END' 'VEVENT' {
        let mut tags = token!($lexer, $14)
            .replace("\r\n ", "")
            .split("\\n")
            .filter_map(|s| {
                if s.is_empty() {
                    return None;
                }
                Some(s.to_owned())
            })
            .collect::<VecDeque<String>>();
        let opt_e = tags.pop_front();
        tags.pop_back();
        if let Some(e) = opt_e {
            tags.push_back(e);
        }
        Ok(Event {
            start: NaiveDateTime::parse_from_str(token!($lexer, $6), DATE_F)
                .map_err(|_| ICalError::DateParse)?
                .and_utc(),
            end: NaiveDateTime::parse_from_str(token!($lexer, $8), DATE_F)
                .map_err(|_| ICalError::DateParse)?
                .and_utc(),
            summary: token!($lexer, $10).replace("\r\n", ""),
            location: token!($lexer, $12).replace("\r\n", ""),
            teacher: tags.pop_front(),
            tags,
        })
    }
    ;
%%
use crate::ical::{
    Event,
    ICal,
    EventFilter,
    error::{Result, ICalError}
};
use chrono::NaiveDateTime;
use std::collections::{VecDeque, BTreeMap};

const DATE_F: &'static str = "%Y%m%dT%H%M%SZ";

macro_rules! token {
    ($lexer:expr, $t:expr) => {
        $lexer.span_str($t.map_err(|_| ICalError::TokenParse)?.span())
    }
}