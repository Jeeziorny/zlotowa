use crate::models::{ClassificationRule, ClassificationSource, ParsedExpense};
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClassifyError {
    #[error("Classification failed: {0}")]
    Failed(String),
    #[error("LLM not configured")]
    LlmNotConfigured,
}

/// Result from a single classifier in the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub category: String,
    pub source: ClassificationSource,
    pub confidence: f64,
}

/// A classifier that can assign categories to expenses.
pub trait Classifier: Send + Sync {
    /// Human-readable name of this classifier.
    fn name(&self) -> &str;

    /// Priority in the pipeline. Lower = tried first.
    fn priority(&self) -> u32;

    /// Try to classify a single expense. Returns None if this classifier
    /// can't determine the category.
    fn classify(&self, expense: &ParsedExpense) -> Result<Option<ClassificationResult>, ClassifyError>;
}

/// Classifier that uses regex rules from the classification database.
pub struct RegexClassifier {
    rules: Vec<(Regex, String)>,
}

impl RegexClassifier {
    pub fn from_rules(rules: &[ClassificationRule]) -> Self {
        let compiled: Vec<(Regex, String)> = rules
            .iter()
            .filter_map(|r| {
                Regex::new(&r.pattern)
                    .ok()
                    .map(|re| (re, r.category.clone()))
            })
            .collect();
        Self { rules: compiled }
    }
}

impl Classifier for RegexClassifier {
    fn name(&self) -> &str {
        "Regex Database"
    }

    fn priority(&self) -> u32 {
        10
    }

    fn classify(&self, expense: &ParsedExpense) -> Result<Option<ClassificationResult>, ClassifyError> {
        for (regex, category) in &self.rules {
            if regex.is_match(&expense.title) {
                return Ok(Some(ClassificationResult {
                    category: category.clone(),
                    source: ClassificationSource::Database,
                    confidence: 1.0,
                }));
            }
        }
        Ok(None)
    }
}

/// Run expenses through the classification pipeline.
/// Classifiers are tried in priority order; first match wins.
pub fn classify_pipeline(
    expenses: &[ParsedExpense],
    classifiers: &[Box<dyn Classifier>],
) -> Vec<(ParsedExpense, Option<ClassificationResult>)> {
    let mut sorted: Vec<&Box<dyn Classifier>> = classifiers.iter().collect();
    sorted.sort_by_key(|c| c.priority());

    expenses
        .iter()
        .map(|expense| {
            let result = sorted
                .iter()
                .find_map(|c| c.classify(expense).ok().flatten());
            (expense.clone(), result)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ClassificationRule;
    use chrono::NaiveDate;

    fn parsed(title: &str) -> ParsedExpense {
        ParsedExpense {
            title: title.to_string(),
            amount: 10.0,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        }
    }

    // ── RegexClassifier ──

    #[test]
    fn regex_classifier_matches_exact() {
        let rules = vec![ClassificationRule {
            id: None,
            pattern: "(?i)starbucks".to_string(),
            category: "Coffee".to_string(),
        }];
        let classifier = RegexClassifier::from_rules(&rules);

        let result = classifier.classify(&parsed("Starbucks")).unwrap();
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.category, "Coffee");
        assert_eq!(r.source, ClassificationSource::Database);
        assert_eq!(r.confidence, 1.0);
    }

    #[test]
    fn regex_classifier_matches_partial() {
        let rules = vec![ClassificationRule {
            id: None,
            pattern: "(?i)grocery".to_string(),
            category: "Food".to_string(),
        }];
        let classifier = RegexClassifier::from_rules(&rules);

        let result = classifier.classify(&parsed("Big Grocery Store")).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().category, "Food");
    }

    #[test]
    fn regex_classifier_no_match() {
        let rules = vec![ClassificationRule {
            id: None,
            pattern: "starbucks".to_string(),
            category: "Coffee".to_string(),
        }];
        let classifier = RegexClassifier::from_rules(&rules);

        let result = classifier.classify(&parsed("Amazon")).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn regex_classifier_first_match_wins() {
        let rules = vec![
            ClassificationRule { id: None, pattern: "shop".to_string(), category: "Shopping".to_string() },
            ClassificationRule { id: None, pattern: "coffee shop".to_string(), category: "Coffee".to_string() },
        ];
        let classifier = RegexClassifier::from_rules(&rules);

        let result = classifier.classify(&parsed("coffee shop")).unwrap().unwrap();
        // "shop" matches first since rules are iterated in order
        assert_eq!(result.category, "Shopping");
    }

    #[test]
    fn regex_classifier_skips_invalid_patterns() {
        let rules = vec![
            ClassificationRule { id: None, pattern: "[invalid".to_string(), category: "Bad".to_string() },
            ClassificationRule { id: None, pattern: "valid".to_string(), category: "Good".to_string() },
        ];
        let classifier = RegexClassifier::from_rules(&rules);

        let result = classifier.classify(&parsed("valid expense")).unwrap().unwrap();
        assert_eq!(result.category, "Good");
    }

    #[test]
    fn regex_classifier_empty_rules() {
        let classifier = RegexClassifier::from_rules(&[]);
        let result = classifier.classify(&parsed("anything")).unwrap();
        assert!(result.is_none());
    }

    // ── Pipeline ──

    #[test]
    fn pipeline_returns_results_for_each_expense() {
        let rules = vec![ClassificationRule {
            id: None,
            pattern: "coffee".to_string(),
            category: "Drinks".to_string(),
        }];
        let classifiers: Vec<Box<dyn Classifier>> = vec![Box::new(RegexClassifier::from_rules(&rules))];

        let expenses = vec![parsed("coffee shop"), parsed("gas station")];
        let results = classify_pipeline(&expenses, &classifiers);

        assert_eq!(results.len(), 2);
        assert!(results[0].1.is_some());
        assert_eq!(results[0].1.as_ref().unwrap().category, "Drinks");
        assert!(results[1].1.is_none());
    }

    #[test]
    fn pipeline_with_no_classifiers() {
        let expenses = vec![parsed("anything")];
        let results = classify_pipeline(&expenses, &[]);
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_none());
    }

    #[test]
    fn pipeline_with_empty_expenses() {
        let classifiers: Vec<Box<dyn Classifier>> = vec![Box::new(RegexClassifier::from_rules(&[]))];
        let results = classify_pipeline(&[], &classifiers);
        assert!(results.is_empty());
    }

    #[test]
    fn regex_classifier_name_and_priority() {
        let classifier = RegexClassifier::from_rules(&[]);
        assert_eq!(classifier.name(), "Regex Database");
        assert_eq!(classifier.priority(), 10);
    }
}
