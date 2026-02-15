use crate::models::{ClassificationSource, ParsedExpense};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClassifyError {
    #[error("Classification failed: {0}")]
    Failed(String),
    #[error("LLM not configured")]
    LlmNotConfigured,
}

/// Result from a single classifier in the pipeline.
#[derive(Debug, Clone)]
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
