"""Tests for provider adapters and the provider factory."""

from __future__ import annotations

from typing import Any
from unittest.mock import AsyncMock, MagicMock

import pytest

from eaml_runtime.errors import EamlConfigError, EamlProviderError
from eaml_runtime.providers import get_provider
from eaml_runtime.providers.anthropic import AnthropicProvider
from eaml_runtime.providers.ollama import OllamaProvider
from eaml_runtime.providers.openai import OpenAIProvider


# --- Provider selection tests ---


def test_get_provider_anthropic() -> None:
    provider = get_provider("anthropic")
    assert isinstance(provider, AnthropicProvider)


def test_get_provider_openai() -> None:
    provider = get_provider("openai")
    assert isinstance(provider, OpenAIProvider)


def test_get_provider_ollama() -> None:
    provider = get_provider("ollama")
    assert isinstance(provider, OllamaProvider)


def test_get_provider_unknown_raises() -> None:
    with pytest.raises(EamlConfigError, match="Unknown provider"):
        get_provider("unknown")


def test_get_provider_caches() -> None:
    p1 = get_provider("ollama")
    p2 = get_provider("ollama")
    assert p1 is p2


# --- API key handling tests ---


def test_anthropic_missing_api_key(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("ANTHROPIC_API_KEY", raising=False)
    provider = AnthropicProvider()
    provider._client = None  # Reset any cached client
    with pytest.raises(EamlConfigError, match="ANTHROPIC_API_KEY"):
        provider._get_client()


def test_openai_missing_api_key(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("OPENAI_API_KEY", raising=False)
    provider = OpenAIProvider()
    provider._client = None
    with pytest.raises(EamlConfigError, match="OPENAI_API_KEY"):
        provider._get_client()


# --- Anthropic provider tests ---


@pytest.mark.asyncio
async def test_anthropic_send_prompt(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("ANTHROPIC_API_KEY", "test-key")

    # Create mock response
    mock_text_block = MagicMock()
    mock_text_block.text = '{"sentiment": "positive"}'
    mock_response = MagicMock()
    mock_response.content = [mock_text_block]

    mock_client = MagicMock()
    mock_client.messages = MagicMock()
    mock_client.messages.create = AsyncMock(return_value=mock_response)

    provider = AnthropicProvider()
    provider._client = mock_client

    messages = [
        {"role": "system", "content": "Be helpful"},
        {"role": "user", "content": "Analyze this"},
    ]
    result = await provider.send_prompt(messages, "anthropic/claude-3-haiku-20240307")

    assert result == '{"sentiment": "positive"}'
    call_kwargs = mock_client.messages.create.call_args
    # System messages extracted to system param
    assert "system" in call_kwargs.kwargs or "system" in (
        call_kwargs[1] if len(call_kwargs) > 1 else {}
    )
    create_kwargs: dict[str, Any] = call_kwargs.kwargs if call_kwargs.kwargs else call_kwargs[1]
    assert create_kwargs["model"] == "claude-3-haiku-20240307"  # Prefix stripped
    assert create_kwargs["max_tokens"] == 4096
    # Only non-system messages in messages param
    assert all(m["role"] != "system" for m in create_kwargs["messages"])


@pytest.mark.asyncio
async def test_anthropic_json_instruction(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("ANTHROPIC_API_KEY", "test-key")

    mock_text_block = MagicMock()
    mock_text_block.text = "{}"
    mock_response = MagicMock()
    mock_response.content = [mock_text_block]

    mock_client = MagicMock()
    mock_client.messages = MagicMock()
    mock_client.messages.create = AsyncMock(return_value=mock_response)

    provider = AnthropicProvider()
    provider._client = mock_client

    messages = [
        {"role": "system", "content": "Be helpful"},
        {"role": "user", "content": "Hello"},
    ]
    await provider.send_prompt(messages, "claude-3")

    create_kwargs: dict[str, Any] = mock_client.messages.create.call_args.kwargs
    assert "Respond with valid JSON only." in create_kwargs["system"]
    assert "Be helpful" in create_kwargs["system"]


@pytest.mark.asyncio
async def test_anthropic_no_system_messages(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("ANTHROPIC_API_KEY", "test-key")

    mock_text_block = MagicMock()
    mock_text_block.text = "{}"
    mock_response = MagicMock()
    mock_response.content = [mock_text_block]

    mock_client = MagicMock()
    mock_client.messages = MagicMock()
    mock_client.messages.create = AsyncMock(return_value=mock_response)

    provider = AnthropicProvider()
    provider._client = mock_client

    messages = [{"role": "user", "content": "Hello"}]
    await provider.send_prompt(messages, "claude-3")

    create_kwargs: dict[str, Any] = mock_client.messages.create.call_args.kwargs
    assert create_kwargs["system"] == "Respond with valid JSON only."


@pytest.mark.asyncio
async def test_anthropic_error_wrapped(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("ANTHROPIC_API_KEY", "test-key")

    mock_client = MagicMock()
    mock_client.messages = MagicMock()
    mock_client.messages.create = AsyncMock(side_effect=RuntimeError("API down"))

    provider = AnthropicProvider()
    provider._client = mock_client

    with pytest.raises(EamlProviderError, match="anthropic") as exc_info:
        await provider.send_prompt([{"role": "user", "content": "Hello"}], "claude-3")
    assert exc_info.value.provider == "anthropic"


# --- OpenAI provider tests ---


@pytest.mark.asyncio
async def test_openai_send_prompt(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("OPENAI_API_KEY", "test-key")

    mock_message = MagicMock()
    mock_message.content = '{"result": "ok"}'
    mock_choice = MagicMock()
    mock_choice.message = mock_message
    mock_response = MagicMock()
    mock_response.choices = [mock_choice]

    mock_client = MagicMock()
    mock_client.chat = MagicMock()
    mock_client.chat.completions = MagicMock()
    mock_client.chat.completions.create = AsyncMock(return_value=mock_response)

    provider = OpenAIProvider()
    provider._client = mock_client

    messages = [{"role": "user", "content": "Hello"}]
    result = await provider.send_prompt(messages, "openai/gpt-4o-mini")

    assert result == '{"result": "ok"}'
    create_kwargs: dict[str, Any] = mock_client.chat.completions.create.call_args.kwargs
    assert create_kwargs["model"] == "gpt-4o-mini"  # Prefix stripped
    assert create_kwargs["response_format"] == {"type": "json_object"}


@pytest.mark.asyncio
async def test_openai_error_wrapped(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("OPENAI_API_KEY", "test-key")

    mock_client = MagicMock()
    mock_client.chat = MagicMock()
    mock_client.chat.completions = MagicMock()
    mock_client.chat.completions.create = AsyncMock(side_effect=RuntimeError("rate limited"))

    provider = OpenAIProvider()
    provider._client = mock_client

    with pytest.raises(EamlProviderError, match="openai") as exc_info:
        await provider.send_prompt([{"role": "user", "content": "Hello"}], "gpt-4o")
    assert exc_info.value.provider == "openai"


# --- Ollama provider tests ---


@pytest.mark.asyncio
async def test_ollama_send_prompt() -> None:
    mock_response = MagicMock()
    mock_response.json.return_value = {"choices": [{"message": {"content": '{"answer": 42}'}}]}
    mock_response.raise_for_status = MagicMock()

    mock_client = MagicMock()
    mock_client.post = AsyncMock(return_value=mock_response)

    provider = OllamaProvider()
    provider._client = mock_client

    messages = [{"role": "user", "content": "Hello"}]
    result = await provider.send_prompt(messages, "ollama/llama3")

    assert result == '{"answer": 42}'
    call_args = mock_client.post.call_args
    assert call_args[0][0] == "http://localhost:11434/v1/chat/completions"
    payload = call_args.kwargs["json"]
    assert payload["model"] == "llama3"  # Prefix stripped
    assert payload["stream"] is False
    assert payload["response_format"] == {"type": "json_object"}


@pytest.mark.asyncio
async def test_ollama_custom_base_url(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("OLLAMA_BASE_URL", "http://gpu-server:11434")

    mock_response = MagicMock()
    mock_response.json.return_value = {"choices": [{"message": {"content": "{}"}}]}
    mock_response.raise_for_status = MagicMock()

    mock_client = MagicMock()
    mock_client.post = AsyncMock(return_value=mock_response)

    provider = OllamaProvider()
    provider._client = mock_client

    await provider.send_prompt([{"role": "user", "content": "Hi"}], "llama3")

    call_args = mock_client.post.call_args
    assert call_args[0][0] == "http://gpu-server:11434/v1/chat/completions"


@pytest.mark.asyncio
async def test_ollama_error_wrapped() -> None:
    mock_client = MagicMock()
    mock_client.post = AsyncMock(side_effect=RuntimeError("connection refused"))

    provider = OllamaProvider()
    provider._client = mock_client

    with pytest.raises(EamlProviderError, match="ollama") as exc_info:
        await provider.send_prompt([{"role": "user", "content": "Hello"}], "llama3")
    assert exc_info.value.provider == "ollama"


# --- Model ID prefix stripping ---


@pytest.mark.asyncio
async def test_model_id_prefix_stripped_anthropic(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("ANTHROPIC_API_KEY", "test-key")

    mock_text_block = MagicMock()
    mock_text_block.text = "{}"
    mock_response = MagicMock()
    mock_response.content = [mock_text_block]

    mock_client = MagicMock()
    mock_client.messages = MagicMock()
    mock_client.messages.create = AsyncMock(return_value=mock_response)

    provider = AnthropicProvider()
    provider._client = mock_client

    await provider.send_prompt(
        [{"role": "user", "content": "Hi"}],
        "anthropic/claude-3-5-sonnet-20241022",
    )

    create_kwargs: dict[str, Any] = mock_client.messages.create.call_args.kwargs
    assert create_kwargs["model"] == "claude-3-5-sonnet-20241022"


@pytest.mark.asyncio
async def test_model_id_no_prefix() -> None:
    """Model IDs without a prefix should pass through unchanged."""
    mock_response = MagicMock()
    mock_response.json.return_value = {"choices": [{"message": {"content": "{}"}}]}
    mock_response.raise_for_status = MagicMock()

    mock_client = MagicMock()
    mock_client.post = AsyncMock(return_value=mock_response)

    provider = OllamaProvider()
    provider._client = mock_client

    await provider.send_prompt([{"role": "user", "content": "Hi"}], "llama3")

    payload = mock_client.post.call_args.kwargs["json"]
    assert payload["model"] == "llama3"
