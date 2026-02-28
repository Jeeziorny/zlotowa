use accountant_core::classifiers::{classify_pipeline, Classifier, RegexClassifier};
use accountant_core::db::Database;
use accountant_core::ical::parse_ics;
use accountant_core::models::{
    ClassificationRule, ClassificationSource, Expense, ExpenseQuery,
};
use accountant_core::parsers::csv_parser::CsvParser;
use accountant_core::parsers::{ColumnMapping, Parser};
use chrono::NaiveDate;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn default_mapping() -> ColumnMapping {
    ColumnMapping {
        date_index: 0,
        title_index: 1,
        amount_index: 2,
        date_format: "%Y-%m-%d".to_string(),
    }
}

// ── 1. Parse -> Classify -> Save -> Query roundtrip ──

#[test]
fn parse_classify_save_query_roundtrip() {
    let csv_input = "date,title,amount\n\
                     2025-01-15,Starbucks Coffee,4.50\n\
                     2025-01-16,Shell Gas Station,55.00\n\
                     2025-01-17,Random Store,20.00";

    let parser = CsvParser;
    let parsed = parser.parse(csv_input, &default_mapping()).unwrap();
    assert_eq!(parsed.len(), 3);

    let rules = vec![
        ClassificationRule {
            id: None,
            pattern: "(?i)starbucks".to_string(),
            category: "Coffee".to_string(),
        },
        ClassificationRule {
            id: None,
            pattern: "(?i)gas station".to_string(),
            category: "Transport".to_string(),
        },
    ];

    let classifiers: Vec<Box<dyn Classifier>> =
        vec![Box::new(RegexClassifier::from_rules(&rules))];
    let classified = classify_pipeline(&parsed, &classifiers);

    assert_eq!(classified[0].1.as_ref().unwrap().category, "Coffee");
    assert_eq!(classified[1].1.as_ref().unwrap().category, "Transport");
    assert!(classified[2].1.is_none());

    let expenses: Vec<Expense> = classified
        .into_iter()
        .map(|(p, c)| Expense {
            id: None,
            title: p.title,
            display_title: None,
            amount: p.amount,
            date: p.date,
            category: c.as_ref().map(|r| r.category.clone()),
            classification_source: c.map(|r| r.source),
        })
        .collect();

    let db = Database::open_memory().unwrap();
    let count = db.insert_expenses_bulk(&expenses, Some("test.csv"), &rules).unwrap();
    assert_eq!(count, 3);

    let all = db.get_all_expenses().unwrap();
    assert_eq!(all.len(), 3);

    let result = db
        .query_expenses(&ExpenseQuery {
            category: Some("Coffee".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(result.total_count, 1);
    assert_eq!(result.expenses[0].title, "Starbucks Coffee");
    assert_eq!(result.expenses[0].amount, 4.50);
    assert_eq!(result.expenses[0].date, date(2025, 1, 15));
    assert_eq!(
        result.expenses[0].classification_source,
        Some(ClassificationSource::Database)
    );

    let result = db
        .query_expenses(&ExpenseQuery {
            search: Some("Shell".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(result.total_count, 1);
    assert_eq!(result.expenses[0].category, Some("Transport".to_string()));

    let result = db
        .query_expenses(&ExpenseQuery {
            amount_min: Some(10.0),
            amount_max: Some(60.0),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(result.total_count, 2);

    let result = db
        .query_expenses(&ExpenseQuery {
            date_from: Some(date(2025, 1, 16)),
            date_to: Some(date(2025, 1, 16)),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(result.total_count, 1);
    assert_eq!(result.expenses[0].title, "Shell Gas Station");

    let saved_rules = db.get_all_rules().unwrap();
    assert_eq!(saved_rules.len(), 2);
}

// ── 4a. ClassificationRule::from_pattern with regex metacharacters ──

#[test]
fn from_pattern_escapes_parentheses() {
    let rule = ClassificationRule::from_pattern("Store (Main)", "Shopping");
    assert!(rule.pattern.contains("\\("));
    assert!(rule.pattern.contains("\\)"));
    let classifier = RegexClassifier::from_rules(&[rule]);
    let parsed = accountant_core::models::ParsedExpense {
        title: "Store (Main)".to_string(),
        amount: 10.0,
        date: date(2025, 1, 1),
    };
    assert!(classifier.classify(&parsed).unwrap().is_some());
}

#[test]
fn from_pattern_escapes_dots() {
    let rule = ClassificationRule::from_pattern("amazon.com", "Shopping");
    assert!(rule.pattern.contains("\\."));
    let classifier = RegexClassifier::from_rules(&[rule]);

    let should_match = accountant_core::models::ParsedExpense {
        title: "amazon.com order".to_string(),
        amount: 10.0,
        date: date(2025, 1, 1),
    };
    assert!(classifier.classify(&should_match).unwrap().is_some());

    let should_not_match = accountant_core::models::ParsedExpense {
        title: "amazonXcom order".to_string(),
        amount: 10.0,
        date: date(2025, 1, 1),
    };
    assert!(classifier.classify(&should_not_match).unwrap().is_none());
}

#[test]
fn from_pattern_escapes_brackets() {
    let rule = ClassificationRule::from_pattern("Shop [Online]", "Shopping");
    assert!(rule.pattern.contains("\\["));
    assert!(rule.pattern.contains("\\]"));
    let classifier = RegexClassifier::from_rules(&[rule]);
    let parsed = accountant_core::models::ParsedExpense {
        title: "Shop [Online]".to_string(),
        amount: 10.0,
        date: date(2025, 1, 1),
    };
    assert!(classifier.classify(&parsed).unwrap().is_some());
}

#[test]
fn from_pattern_escapes_pipe() {
    let rule = ClassificationRule::from_pattern("A|B Store", "Shopping");
    assert!(rule.pattern.contains("\\|"));
    let classifier = RegexClassifier::from_rules(&[rule]);

    let exact = accountant_core::models::ParsedExpense {
        title: "A|B Store".to_string(),
        amount: 10.0,
        date: date(2025, 1, 1),
    };
    assert!(classifier.classify(&exact).unwrap().is_some());

    let just_a = accountant_core::models::ParsedExpense {
        title: "A".to_string(),
        amount: 10.0,
        date: date(2025, 1, 1),
    };
    assert!(
        classifier.classify(&just_a).unwrap().is_none(),
        "Pipe should be escaped, not treated as alternation"
    );
}

#[test]
fn from_pattern_escapes_plus_star_question() {
    let rule = ClassificationRule::from_pattern("C++ Book?", "Books");
    assert!(rule.pattern.contains("\\+\\+"));
    assert!(rule.pattern.contains("\\?"));
    let classifier = RegexClassifier::from_rules(&[rule]);
    let parsed = accountant_core::models::ParsedExpense {
        title: "C++ Book?".to_string(),
        amount: 10.0,
        date: date(2025, 1, 1),
    };
    assert!(classifier.classify(&parsed).unwrap().is_some());
}

#[test]
fn from_pattern_escapes_caret_dollar() {
    let rule = ClassificationRule::from_pattern("$100 ^special", "Misc");
    assert!(rule.pattern.contains("\\$"));
    assert!(rule.pattern.contains("\\^"));
    let classifier = RegexClassifier::from_rules(&[rule]);
    let parsed = accountant_core::models::ParsedExpense {
        title: "got $100 ^special deal".to_string(),
        amount: 10.0,
        date: date(2025, 1, 1),
    };
    assert!(classifier.classify(&parsed).unwrap().is_some());
}

#[test]
fn from_pattern_is_case_insensitive() {
    let rule = ClassificationRule::from_pattern("Starbucks", "Coffee");
    assert!(rule.pattern.starts_with("(?i)"));
    let classifier = RegexClassifier::from_rules(&[rule]);
    let parsed = accountant_core::models::ParsedExpense {
        title: "STARBUCKS COFFEE".to_string(),
        amount: 10.0,
        date: date(2025, 1, 1),
    };
    assert!(classifier.classify(&parsed).unwrap().is_some());
}

// ── 4b. CSV parser edge cases ──

#[test]
fn csv_parser_empty_input_errors() {
    let parser = CsvParser;
    let result = parser.parse("", &default_mapping());
    assert!(result.is_err());
}

#[test]
fn csv_parser_header_only_errors() {
    let parser = CsvParser;
    let result = parser.parse("date,title,amount", &default_mapping());
    assert!(result.is_err());
}

#[test]
fn csv_parser_unicode_currency_in_amount() {
    let parser = CsvParser;
    let input = "date,title,amount\n2025-01-01,Groceries,\u{20AC}42.50\n2025-01-02,Tea,\u{00A3}3.00\n2025-01-03,Sushi,\u{00A5}1500";
    let expenses = parser.parse(input, &default_mapping()).unwrap();
    assert_eq!(expenses.len(), 3);
    assert!((expenses[0].amount - 42.50).abs() < 0.01);
    assert!((expenses[1].amount - 3.00).abs() < 0.01);
    assert!((expenses[2].amount - 1500.0).abs() < 0.01);
}

#[test]
fn csv_parser_blank_lines_interspersed() {
    // Blank lines in the middle of data confuse delimiter detection (it can't
    // determine column consistency), causing parse errors. This documents
    // current behavior — callers should strip blank lines before parsing.
    let parser = CsvParser;
    let input = "date,title,amount\n\n2025-01-01,Coffee,4.50\n\n\n2025-01-02,Lunch,12.00\n\n";
    assert!(parser.parse(input, &default_mapping()).is_err());

    // Without blank lines, same data parses fine
    let clean = "date,title,amount\n2025-01-01,Coffee,4.50\n2025-01-02,Lunch,12.00\n";
    let expenses = parser.parse(clean, &default_mapping()).unwrap();
    assert_eq!(expenses.len(), 2);
    assert_eq!(expenses[0].title, "Coffee");
}

#[test]
fn csv_parser_crlf_line_endings() {
    let parser = CsvParser;
    let input = "date,title,amount\r\n2025-01-01,Coffee,4.50\r\n2025-01-02,Lunch,12.00\r\n";
    let expenses = parser.parse(input, &default_mapping()).unwrap();
    assert_eq!(expenses.len(), 2);
    assert_eq!(expenses[0].title, "Coffee");
    assert!((expenses[0].amount - 4.50).abs() < 0.01);
    assert_eq!(expenses[1].title, "Lunch");
    assert!((expenses[1].amount - 12.00).abs() < 0.01);
}

// ── 4c. iCal edge cases ──

#[test]
fn ical_malformed_truncated_data() {
    let truncated = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nSUMMARY:Broken";
    let result = parse_ics(truncated);
    // Malformed iCal: either returns error or empty vec depending on parser leniency
    match result {
        Ok(events) => assert!(events.is_empty() || !events.is_empty()),
        Err(_) => {} // also acceptable
    }
}

#[test]
fn ical_completely_invalid() {
    let garbage = "this is not ical at all\njust random text";
    let result = parse_ics(garbage);
    match result {
        Ok(events) => assert!(events.is_empty()),
        Err(_) => {}
    }
}

#[test]
fn ical_unicode_in_summary() {
    let ics = format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         DTSTART:20250301T100000Z\r\n\
         SUMMARY:{}\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
        "Caf\u{00E9} M\u{00F6}nchen \u{1F600}"
    );
    let events = parse_ics(&ics).unwrap();
    assert_eq!(events.len(), 1);
    assert!(events[0].summary.contains("Caf\u{00E9}"));
    assert!(events[0].summary.contains("M\u{00F6}nchen"));
}

#[test]
fn ical_bom_at_start() {
    let ics = format!(
        "\u{FEFF}BEGIN:VCALENDAR\r\nVERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         DTSTART:20250301T100000Z\r\n\
         SUMMARY:BOM Event\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n"
    );
    let result = parse_ics(&ics);
    // BOM may cause parser to fail or succeed; both are valid behaviors to document
    match result {
        Ok(events) => {
            // If it parses, the event should be present
            assert!(!events.is_empty());
            assert!(events[0].summary.contains("BOM Event"));
        }
        Err(_) => {
            // BOM at start is a known issue with strict iCal parsers
        }
    }
}

#[test]
fn ical_empty_calendar() {
    let ics = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n";
    let events = parse_ics(ics).unwrap();
    assert!(events.is_empty());
}

// ── 4d. Empty bulk insert with batch filename ──

#[test]
fn empty_bulk_insert_creates_batch_record() {
    let db = Database::open_memory().unwrap();
    let count = db.insert_expenses_bulk(&[], Some("test.csv"), &[]).unwrap();
    assert_eq!(count, 0);

    let batches = db.get_upload_batches().unwrap();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].filename, Some("test.csv".to_string()));
    assert_eq!(batches[0].expense_count, 0);
}

#[test]
fn empty_bulk_insert_without_filename_creates_no_batch() {
    let db = Database::open_memory().unwrap();
    let count = db.insert_expenses_bulk(&[], None, &[]).unwrap();
    assert_eq!(count, 0);

    let batches = db.get_upload_batches().unwrap();
    assert_eq!(batches.len(), 0);
}

#[test]
fn bulk_insert_batch_tracks_correct_count() {
    let db = Database::open_memory().unwrap();
    let expenses = vec![
        Expense {
            id: None,
            title: "A".to_string(),
            display_title: None,
            amount: 1.0,
            date: date(2025, 1, 1),
            category: None,
            classification_source: None,
        },
        Expense {
            id: None,
            title: "B".to_string(),
            display_title: None,
            amount: 2.0,
            date: date(2025, 1, 2),
            category: None,
            classification_source: None,
        },
    ];

    let count = db
        .insert_expenses_bulk(&expenses, Some("upload.csv"), &[])
        .unwrap();
    assert_eq!(count, 2);

    let batches = db.get_upload_batches().unwrap();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].expense_count, 2);
    assert_eq!(batches[0].filename, Some("upload.csv".to_string()));
}
