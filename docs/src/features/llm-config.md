# LLM Configuration

LLM classification is **optional**. The app works fully without it — you can classify everything manually or rely on learned rules.

## Setup

Go to **Settings** in the sidebar and configure:

- **Provider** — choose between OpenAI, Anthropic, or Ollama (local)
- **API Key** — your provider's API key (or endpoint URL for Ollama)

Click **Save** to validate and store the configuration. The app checks your key against the provider's API before saving — if the key is invalid, you'll see an error immediately.

You can also click **Test Connection** to verify your setup without saving changes.

Your API key is stored in the local database on your machine.

## Supported providers

| Provider | Key format | Notes |
|---|---|---|
| OpenAI | `sk-...` | Uses the OpenAI API |
| Anthropic | `sk-ant-...` | Uses the Anthropic API |
| Ollama | `http://localhost:11434` | Runs locally, no API key needed |

## Clearing configuration

Click **Clear** in Settings to remove the stored provider and API key. The app will fall back to rule-based and manual classification only.
