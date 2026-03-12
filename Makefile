.PHONY: up purge-data fresh-expenses

up:
	npm run tauri dev

DB_PATH := $(HOME)/Library/Application Support/zlotowa/zlotowa.db

## Wipe all rows from every table, keeping schema intact
purge-data:
	sqlite3 "$(DB_PATH)" " \
		DELETE FROM expenses; \
		DELETE FROM classification_rules; \
		DELETE FROM config WHERE key NOT IN ('llm_provider', 'llm_api_key'); \
		DELETE FROM budgets; \
		DELETE FROM budget_categories; \
		DELETE FROM upload_batches; \
		VACUUM; \
	"
	@echo "All data purged from $(DB_PATH)"

## Wipe expenses and batches only, keep categories, rules, config, budgets
fresh-expenses:
	sqlite3 "$(DB_PATH)" " \
		DELETE FROM expenses; \
		DELETE FROM upload_batches; \
		VACUUM; \
	"
	@echo "Expenses cleared, rules & categories kept in $(DB_PATH)"
