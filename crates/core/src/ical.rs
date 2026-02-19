use chrono::NaiveDate;
use ical::parser::ical::component::IcalCalendar;
use ical::IcalParser;
use std::io::BufReader;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IcalError {
    #[error("Failed to parse iCal data: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone)]
pub struct ParsedCalendarEvent {
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub all_day: bool,
}

/// Parse an .ics file content into calendar events.
///
/// Handles three DTSTART formats:
/// - `DTSTART:20250227T160000Z` — UTC datetime
/// - `DTSTART;VALUE=DATE:20241229` — all-day event
/// - `DTSTART;TZID=Europe/Warsaw:20251118T180000` — timezone-qualified
///
/// Skips events without SUMMARY or DTSTART. Ignores RRULE.
pub fn parse_ics(input: &str) -> Result<Vec<ParsedCalendarEvent>, IcalError> {
    let reader = BufReader::new(input.as_bytes());
    let parser = IcalParser::new(reader);

    let mut events = Vec::new();

    for calendar_result in parser {
        let calendar: IcalCalendar =
            calendar_result.map_err(|e| IcalError::ParseError(e.to_string()))?;

        for event in calendar.events {
            let mut summary: Option<String> = None;
            let mut description: Option<String> = None;
            let mut location: Option<String> = None;
            let mut dtstart: Option<String> = None;
            let mut dtstart_params: Vec<(String, Vec<String>)> = Vec::new();
            let mut dtend: Option<String> = None;
            let mut dtend_params: Vec<(String, Vec<String>)> = Vec::new();

            for prop in &event.properties {
                match prop.name.as_str() {
                    "SUMMARY" => summary = prop.value.clone(),
                    "DESCRIPTION" => description = prop.value.clone(),
                    "LOCATION" => location = prop.value.clone(),
                    "DTSTART" => {
                        dtstart = prop.value.clone();
                        if let Some(params) = &prop.params {
                            dtstart_params = params.clone();
                        }
                    }
                    "DTEND" => {
                        dtend = prop.value.clone();
                        if let Some(params) = &prop.params {
                            dtend_params = params.clone();
                        }
                    }
                    _ => {}
                }
            }

            // Skip events without summary or start date
            let summary = match summary {
                Some(s) if !s.trim().is_empty() => s,
                _ => continue,
            };
            let dtstart_str = match dtstart {
                Some(s) => s,
                None => continue,
            };

            let (start_date, all_day) = match parse_ical_date(&dtstart_str, &dtstart_params) {
                Some(result) => result,
                None => continue,
            };

            let end_date = dtend
                .as_deref()
                .and_then(|s| parse_ical_date(s, &dtend_params))
                .map(|(d, _)| d);

            events.push(ParsedCalendarEvent {
                summary,
                description,
                location,
                start_date,
                end_date,
                all_day,
            });
        }
    }

    Ok(events)
}

/// Filter events to those starting in a given year/month.
pub fn filter_events_by_month(
    events: &[ParsedCalendarEvent],
    year: i32,
    month: u32,
) -> Vec<ParsedCalendarEvent> {
    events
        .iter()
        .filter(|e| e.start_date.year() == year && e.start_date.month() == month)
        .cloned()
        .collect()
}

/// Filter events to those starting within a date range (inclusive).
pub fn filter_events_by_date_range(
    events: &[ParsedCalendarEvent],
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Vec<ParsedCalendarEvent> {
    events
        .iter()
        .filter(|e| e.start_date >= start_date && e.start_date <= end_date)
        .cloned()
        .collect()
}

use chrono::Datelike;

/// Parse an iCal date/datetime string into a NaiveDate and all_day flag.
///
/// Supported formats:
/// - `20250227T160000Z` — UTC datetime → (2025-02-27, false)
/// - `20241229` with VALUE=DATE param — all-day → (2024-12-29, true)
/// - `20251118T180000` with TZID param — local datetime → (2025-11-18, false)
fn parse_ical_date(
    value: &str,
    params: &[(String, Vec<String>)],
) -> Option<(NaiveDate, bool)> {
    let is_date_only = params
        .iter()
        .any(|(k, v)| k == "VALUE" && v.iter().any(|val| val == "DATE"));

    if is_date_only {
        // Pure date: YYYYMMDD
        let date = NaiveDate::parse_from_str(value, "%Y%m%d").ok()?;
        return Some((date, true));
    }

    // DateTime with or without timezone: YYYYMMDDTHHMMSS[Z]
    let date_part = value.split('T').next()?;
    let date = NaiveDate::parse_from_str(date_part, "%Y%m%d").ok()?;
    Some((date, false))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ics(events_block: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n{}\r\nEND:VCALENDAR\r\n",
            events_block
        )
    }

    fn make_event(dtstart: &str, summary: &str) -> String {
        format!(
            "BEGIN:VEVENT\r\n{}\r\nSUMMARY:{}\r\nEND:VEVENT",
            dtstart, summary
        )
    }

    #[test]
    fn parse_utc_datetime() {
        let ics = make_ics(&make_event("DTSTART:20250227T160000Z", "Meeting"));
        let events = parse_ics(&ics).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].start_date, NaiveDate::from_ymd_opt(2025, 2, 27).unwrap());
        assert!(!events[0].all_day);
        assert_eq!(events[0].summary, "Meeting");
    }

    #[test]
    fn parse_all_day_event() {
        let ics = make_ics(&make_event("DTSTART;VALUE=DATE:20241229", "Holiday"));
        let events = parse_ics(&ics).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].start_date, NaiveDate::from_ymd_opt(2024, 12, 29).unwrap());
        assert!(events[0].all_day);
    }

    #[test]
    fn parse_timezone_datetime() {
        let ics = make_ics(&make_event(
            "DTSTART;TZID=Europe/Warsaw:20251118T180000",
            "Dinner",
        ));
        let events = parse_ics(&ics).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].start_date, NaiveDate::from_ymd_opt(2025, 11, 18).unwrap());
        assert!(!events[0].all_day);
    }

    #[test]
    fn skip_event_without_summary() {
        let ics = make_ics("BEGIN:VEVENT\r\nDTSTART:20250301T100000Z\r\nEND:VEVENT");
        let events = parse_ics(&ics).unwrap();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn skip_event_without_dtstart() {
        let ics = make_ics("BEGIN:VEVENT\r\nSUMMARY:No Date\r\nEND:VEVENT");
        let events = parse_ics(&ics).unwrap();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn filter_events_by_month_works() {
        let ics = make_ics(&[
            make_event("DTSTART:20250115T100000Z", "Jan Event"),
            make_event("DTSTART:20250301T100000Z", "Mar Event 1"),
            make_event("DTSTART:20250315T100000Z", "Mar Event 2"),
            make_event("DTSTART:20250401T100000Z", "Apr Event"),
        ].join("\r\n"));
        let events = parse_ics(&ics).unwrap();
        assert_eq!(events.len(), 4);

        let march = filter_events_by_month(&events, 2025, 3);
        assert_eq!(march.len(), 2);
        assert_eq!(march[0].summary, "Mar Event 1");
        assert_eq!(march[1].summary, "Mar Event 2");
    }

    #[test]
    fn parse_event_with_description_and_location() {
        let ics = make_ics(
            "BEGIN:VEVENT\r\nDTSTART:20250301T100000Z\r\nSUMMARY:Dentist\r\nDESCRIPTION:Annual checkup\r\nLOCATION:Main St 123\r\nEND:VEVENT",
        );
        let events = parse_ics(&ics).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].description.as_deref(), Some("Annual checkup"));
        assert_eq!(events[0].location.as_deref(), Some("Main St 123"));
    }

    #[test]
    fn parse_event_with_dtend() {
        let ics = make_ics(
            "BEGIN:VEVENT\r\nDTSTART:20250301T100000Z\r\nDTEND:20250301T120000Z\r\nSUMMARY:Meeting\r\nEND:VEVENT",
        );
        let events = parse_ics(&ics).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].end_date, Some(NaiveDate::from_ymd_opt(2025, 3, 1).unwrap()));
    }

    #[test]
    fn empty_input_returns_empty() {
        let ics = make_ics("");
        let events = parse_ics(&ics).unwrap();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn filter_events_by_date_range_works() {
        let ics = make_ics(&[
            make_event("DTSTART:20250115T100000Z", "Jan Event"),
            make_event("DTSTART:20250301T100000Z", "Mar Event 1"),
            make_event("DTSTART:20250315T100000Z", "Mar Event 2"),
            make_event("DTSTART:20250401T100000Z", "Apr Event"),
        ].join("\r\n"));
        let events = parse_ics(&ics).unwrap();
        assert_eq!(events.len(), 4);

        let filtered = filter_events_by_date_range(
            &events,
            NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
        );
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].summary, "Mar Event 1");
        assert_eq!(filtered[1].summary, "Mar Event 2");
    }

    #[test]
    fn multiple_calendars_in_one_file() {
        let ics = format!(
            "{}\r\n{}",
            make_ics(&make_event("DTSTART:20250301T100000Z", "Event A")),
            make_ics(&make_event("DTSTART:20250302T100000Z", "Event B")),
        );
        let events = parse_ics(&ics).unwrap();
        assert_eq!(events.len(), 2);
    }
}
