"""Tests for validate_or_retry and primitive type handling."""

from __future__ import annotations

from typing import Any, Literal

import pytest
from pydantic import BaseModel

from eaml_runtime.errors import EamlValidationError
from eaml_runtime.events import ValidationFailureEvent
from eaml_runtime.providers import Provider
from eaml_runtime.telemetry import configure
from eaml_runtime.validation import validate_or_retry


class Greeting(BaseModel):
    message: str
    word_count: int


class MockProvider(Provider):
    """Test provider returning canned responses."""

    def __init__(self, responses: list[str]) -> None:
        self.responses = list(responses)
        self.calls: list[list[dict[str, str]]] = []
        self._idx = 0

    async def send_prompt(
        self,
        messages: list[dict[str, str]],
        model_id: str,
        *,
        temperature: float | None = None,
        max_tokens: int | None = None,
    ) -> str:
        self.calls.append([dict(m) for m in messages])
        if self._idx < len(self.responses):
            resp = self.responses[self._idx]
            self._idx += 1
            return resp
        return "{}"


class ErrorProvider(Provider):
    """Test provider that always raises."""

    def __init__(self, exc: Exception) -> None:
        self.exc = exc

    async def send_prompt(
        self,
        messages: list[dict[str, str]],
        model_id: str,
        *,
        temperature: float | None = None,
        max_tokens: int | None = None,
    ) -> str:
        raise self.exc


# --- BaseModel validation tests ---


@pytest.mark.asyncio
async def test_validate_or_retry_success() -> None:
    provider = MockProvider(['{"message": "hi", "word_count": 1}'])
    messages: list[dict[str, str]] = [{"role": "user", "content": "hello"}]

    result = await validate_or_retry(provider, messages, "test-model", Greeting)

    assert isinstance(result, Greeting)
    assert result.message == "hi"
    assert result.word_count == 1


@pytest.mark.asyncio
async def test_validate_or_retry_retries_on_invalid_json() -> None:
    provider = MockProvider(["not json", '{"message": "ok", "word_count": 2}'])
    messages: list[dict[str, str]] = [{"role": "user", "content": "hello"}]

    result = await validate_or_retry(provider, messages, "test-model", Greeting)

    assert isinstance(result, Greeting)
    assert result.message == "ok"
    assert len(provider.calls) == 2


@pytest.mark.asyncio
async def test_validate_or_retry_retries_on_schema_mismatch() -> None:
    provider = MockProvider(['{"wrong": "field"}', '{"message": "ok", "word_count": 3}'])
    messages: list[dict[str, str]] = [{"role": "user", "content": "hello"}]

    result = await validate_or_retry(provider, messages, "test-model", Greeting)

    assert isinstance(result, Greeting)
    assert result.word_count == 3


@pytest.mark.asyncio
async def test_validate_or_retry_exhausts_retries() -> None:
    provider = MockProvider(["bad"] * 5)
    messages: list[dict[str, str]] = [{"role": "user", "content": "hello"}]

    with pytest.raises(EamlValidationError) as exc_info:
        await validate_or_retry(
            provider, messages, "test-model", Greeting, max_retries=3
        )

    assert exc_info.value.attempts == 3
    assert exc_info.value.model_id == "test-model"


@pytest.mark.asyncio
async def test_validation_error_contains_all_errors() -> None:
    provider = MockProvider(["bad1", "bad2", "bad3"])
    messages: list[dict[str, str]] = [{"role": "user", "content": "hello"}]

    with pytest.raises(EamlValidationError) as exc_info:
        await validate_or_retry(
            provider, messages, "test-model", Greeting, max_retries=3
        )

    assert len(exc_info.value.errors) == 3


@pytest.mark.asyncio
async def test_validation_error_contains_last_response() -> None:
    provider = MockProvider(["bad1", "bad2", "last_bad"])
    messages: list[dict[str, str]] = [{"role": "user", "content": "hello"}]

    with pytest.raises(EamlValidationError) as exc_info:
        await validate_or_retry(
            provider, messages, "test-model", Greeting, max_retries=3
        )

    assert exc_info.value.last_response == "last_bad"


@pytest.mark.asyncio
async def test_retry_appends_error_message() -> None:
    provider = MockProvider(["bad", '{"message": "ok", "word_count": 1}'])
    messages: list[dict[str, str]] = [{"role": "user", "content": "hello"}]

    await validate_or_retry(provider, messages, "test-model", Greeting)

    # After first failure, an error feedback message was appended
    assert len(messages) == 2
    assert "not valid" in messages[1]["content"].lower() or "error" in messages[1]["content"].lower()
    # Second call should have received the extended messages
    assert len(provider.calls[1]) == 2


# --- Primitive type tests ---


@pytest.mark.asyncio
async def test_validate_primitive_str() -> None:
    provider = MockProvider(['"hello"'])
    messages: list[dict[str, str]] = [{"role": "user", "content": "test"}]

    result = await validate_or_retry(provider, messages, "m", str)

    assert result == "hello"
    assert isinstance(result, str)


@pytest.mark.asyncio
async def test_validate_primitive_int() -> None:
    provider = MockProvider(["42"])
    messages: list[dict[str, str]] = [{"role": "user", "content": "test"}]

    result = await validate_or_retry(provider, messages, "m", int)

    assert result == 42
    assert isinstance(result, int)


@pytest.mark.asyncio
async def test_validate_primitive_float() -> None:
    provider = MockProvider(["3.14"])
    messages: list[dict[str, str]] = [{"role": "user", "content": "test"}]

    result = await validate_or_retry(provider, messages, "m", float)

    assert result == pytest.approx(3.14)
    assert isinstance(result, float)


@pytest.mark.asyncio
async def test_validate_primitive_bool() -> None:
    provider = MockProvider(["true"])
    messages: list[dict[str, str]] = [{"role": "user", "content": "test"}]

    result = await validate_or_retry(provider, messages, "m", bool)

    assert result is True


@pytest.mark.asyncio
async def test_validate_literal() -> None:
    provider = MockProvider(['"high"'])
    messages: list[dict[str, str]] = [{"role": "user", "content": "test"}]

    result = await validate_or_retry(
        provider, messages, "m", Literal["low", "medium", "high"]
    )

    assert result == "high"


@pytest.mark.asyncio
async def test_validate_literal_invalid() -> None:
    provider = MockProvider(['"invalid"'] * 3)
    messages: list[dict[str, str]] = [{"role": "user", "content": "test"}]

    with pytest.raises(EamlValidationError):
        await validate_or_retry(
            provider, messages, "m", Literal["low", "medium", "high"], max_retries=3
        )


# --- Provider error tests ---


@pytest.mark.asyncio
async def test_provider_error_bubbles_up() -> None:
    provider = ErrorProvider(RuntimeError("boom"))
    messages: list[dict[str, str]] = [{"role": "user", "content": "test"}]

    with pytest.raises(RuntimeError, match="boom"):
        await validate_or_retry(provider, messages, "m", Greeting)


# --- Telemetry tests ---


@pytest.mark.asyncio
async def test_validation_failure_event_fired() -> None:
    events: list[Any] = []
    configure(on_validation_failure=events.append)

    provider = MockProvider(["bad", '{"message": "ok", "word_count": 1}'])
    messages: list[dict[str, str]] = [{"role": "user", "content": "test"}]

    await validate_or_retry(
        provider, messages, "test-model", Greeting, provider_name="anthropic"
    )

    assert len(events) == 1
    assert isinstance(events[0], ValidationFailureEvent)
    assert events[0].provider == "anthropic"
    assert events[0].model_id == "test-model"
    assert events[0].attempt == 1
