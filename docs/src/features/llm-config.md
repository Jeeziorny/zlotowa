# LLM Configuration

LLM classification is **optional**. The app works fully without it — you can classify everything manually or rely on learned rules.

When enabled, the LLM acts as a **fallback classifier** — expenses that can't be matched by your existing regex rules get sent to the LLM in a batch for category suggestions.

## What is an API key?

An API key is a secret token that lets 4ccountant send requests to an LLM service on your behalf. The provider uses it to identify your account and bill you for usage. Think of it as a password for programmatic access.

> **Note:** An API key is separate from any product subscription you may have with the same provider. For example, having a Claude Pro subscription does not give you API access — you need to add credits to your API account separately at [console.anthropic.com](https://console.anthropic.com).

## Supported providers

| Provider | Model used | Cost | Key format |
|---|---|---|---|
| **OpenAI** | gpt-4o-mini | Pay-per-use (very cheap) | `sk-...` |
| **Anthropic** | Claude Haiku 4.5 | Pay-per-use (very cheap) | `sk-ant-...` |
| **Ollama** | llama3 | Free (runs locally) | Not needed — enter endpoint URL |

4ccountant uses small, cheap models. Classifying expenses costs fractions of a cent per batch, so even $5 in API credits will last a long time.

## Getting an API key

### OpenAI

1. Create an account at [platform.openai.com](https://platform.openai.com)
2. Go to the **API Keys** section
3. Click **Create new secret key** and copy the `sk-...` string
4. Add billing credits to your account under **Settings > Billing**

### Anthropic

1. Create an account at [console.anthropic.com](https://console.anthropic.com)
2. Go to the **API Keys** section
3. Click **Create Key** and copy the `sk-ant-...` string
4. Add billing credits under **Plans & Billing**

### Ollama (free, local)

No API key or account needed. Ollama runs models on your machine.

1. Install Ollama from [ollama.com](https://ollama.com)
2. Pull a model: `ollama pull llama3`
3. Ollama serves automatically at `http://localhost:11434`
4. In 4ccountant, select **Ollama** and leave the endpoint as the default (or enter a custom URL if you changed it)

## Setup (GUI)

1. Open 4ccountant and go to **Settings** in the sidebar
2. Select your **Provider** from the dropdown
3. Enter your **API Key** (or endpoint URL for Ollama)
4. Click **Save** — the app validates your key against the provider's API before saving. If the key is invalid or your balance is too low, you'll see an error immediately
5. You can also click **Test Connection** to verify your setup without saving

Your API key is stored in the local SQLite database on your machine.

## Setup (CLI)

Run the interactive configuration command:

```bash
4ccountant llm-conf
```

This will prompt you to select a provider and enter your API key. The key is validated before saving.

## How it works

During bulk import (both GUI and CLI), the classification pipeline runs in two stages:

1. **Regex rules** — expenses are matched against your saved classification rules first
2. **LLM fallback** — any remaining unclassified expenses are sent to the LLM in a single batch call, which returns a suggested category for each

If the LLM call fails (network error, expired key, etc.), the expenses simply remain unclassified — it never blocks your import.

## Clearing configuration

Click **Clear** in Settings to remove the stored provider and API key. The app will fall back to rule-based and manual classification only. (The CLI `llm-conf` command always sets a new provider — it has no clear option.)
