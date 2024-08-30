mod ical;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use ical::EventFilter;
use regex::Regex;
use serde::{Deserialize, Serialize};

const URL: &'static str =
    "https://adecampus.univ-rouen.fr/jsp/custom/modules/plannings/anonymous_cal.jsp";
const CALENDARS_REG: &'static str = r"\d+(?:,\d+)*";

#[derive(Deserialize)]
struct CalQuery {
    summary: Option<String>,
    location: Option<String>,
    teacher: Option<String>,
    tags: Option<String>,
    all: Option<bool>,
    calendars: String,
}

#[derive(Serialize)]
struct APIError {
    error: String,
}

macro_rules! query_parse {
    ($query:expr, $field:ident, $reg:ident) => {
        let $reg;
        let $field = if let Some($field) = $query.$field.as_ref() {
            match Regex::new($field) {
                Ok(reg) => {
                    $reg = reg;
                    Some(&$reg)
                }
                Err(err) => {
                    return HttpResponse::BadRequest().json(APIError {
                        error: err.to_string(),
                    })
                }
            }
        } else {
            None
        };
    };
}

async fn index(query: web::Query<CalQuery>) -> impl Responder {
    if !Regex::new(CALENDARS_REG)
        .unwrap()
        .is_match(&query.calendars)
    {
        return HttpResponse::BadRequest().json(APIError {
            error: "Invalid format for calendar id list".to_owned(),
        });
    }
    query_parse!(query, summary, summary_reg);
    query_parse!(query, location, location_reg);
    query_parse!(query, teacher, location_reg);
    query_parse!(query, tags, location_reg);
    let all = query.all.unwrap_or(false);
    let filter = EventFilter {
        summary,
        location,
        teacher,
        tags,
        all,
    };
    let client = reqwest::Client::new();
    let res = match client
        .get(URL)
        .query(&[
            ("resources", query.calendars.as_str()),
            ("calType", "ical"),
            ("projectId", "0"),
            ("nbWeeks", "48"),
        ])
        .send()
        .await
    {
        Ok(res) => match res.text().await {
            Ok(res) => res,
            Err(err) => {
                return HttpResponse::BadRequest().json(APIError {
                    error: err.to_string(),
                });
            }
        },
        Err(err) => {
            return HttpResponse::BadRequest().json(APIError {
                error: err.to_string(),
            });
        }
    };
    match ical::parse(&res, filter) {
        Ok(ical) => HttpResponse::Ok().json(ical),
        Err(err) => HttpResponse::BadRequest().json(APIError {
            error: err.to_string(),
        }),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(index)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
