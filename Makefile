DB_PATH := $(HOME)/Library/Application Support/4ccountant/4ccountant.db

## Wipe all rows from every table, keeping schema intact
purge-data:
	sqlite3 "$(DB_PATH)" " \
		DELETE FROM expenses; \
		DELETE FROM classification_rules; \
		DELETE FROM config WHERE key NOT IN ('llm_provider', 'llm_api_key'); \
		DELETE FROM title_cleanup_rules; \
		DELETE FROM budgets; \
		DELETE FROM budget_categories; \
		DELETE FROM planned_expenses; \
		DELETE FROM calendar_events; \
		DELETE FROM upload_batches; \
		VACUUM; \
	"
	@echo "All data purged from $(DB_PATH)"
