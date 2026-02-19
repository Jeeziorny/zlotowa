# 35 — DB & Core Edge Case Tests

## Problem

Existing tests cover happy paths but miss important edge cases:

### db.rs
- `query_expenses()` — untested for `amount_min > amount_max`, `date_from > date_to`, Unicode search, `limit=0`
- `rename_category()` — untested for empty string, existing name, whitespace
- `merge_categories()` — untested for empty sources, self-merge, duplicate sources
- `delete_category()` — untested for nonexistent replacement, same-name replacement

### classifiers.rs
- `RegexClassifier::from_rules` — all-invalid patterns, duplicate patterns with different categories
- `classify_pipeline` — classifier returning Err, empty classifier list

### llm.rs
- `parse_classification_response` — zero-indexed IDs, empty categories, ID gaps, invalid confidence
- `extract_json_array` — trailing commas, escaped unicode, multiple arrays

### parsers/csv_parser.rs
- `parse_amount` — signed zero, multiple signs, very large exponents
- `split_csv_line` — unmatched quotes, nested quotes

## Scope

- Add targeted edge case tests for each function listed above
- Focus on inputs that could cause panics, silent data loss, or wrong results
