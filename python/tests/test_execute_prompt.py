"""Tests for execute_prompt pipeline, Agent, and ToolMetadata."""

from __future__ import annotations

from typing import Any
from unittest.mock import patch

import pytest

from eaml_runtime.agent import Agent, ToolMetadata
from eaml_runtime.errors import EamlConfigError, EamlProviderError
from eaml_runtime.events import CallEndEvent, CallStartEvent
from eaml_runtime.telemetry import configure
from eaml_runtime.validation import execute_prompt

from tests.helpers import ErrorProvider, Greeting, MockProvider


MODEL = {"provider": "anthropic", "model_id": "test-model", "capabilities": []}


def _patch_provider(provider: Any) -> Any:
    return patch("eaml_runtime.validation.get_provider", return_value=provider)


# --- execute_prompt tests ---


@pytest.mark.asyncio
async def test_execute_prompt_returns_validated_model() -> None:
    mock = MockProvider(['{"message": "hi", "word_count": 1}'])
    with _patch_provider(mock):
        result = await execute_prompt(
            model=MODEL,
            messages=[{"role": "user", "content": "hello"}],
            return_type=Greeting,
        )

    assert isinstance(result, Greeting)
    assert result.message == "hi"


@pytest.mark.asyncio
async def test_execute_prompt_with_primitive_return() -> None:
    mock = MockProvider(['"hello"'])
    with _patch_provider(mock):
        result = await execute_prompt(
            model=MODEL,
            messages=[{"role": "user", "content": "test"}],
            return_type=str,
        )

    assert result == "hello"


@pytest.mark.asyncio
async def test_execute_prompt_fires_call_start_event() -> None:
    events: list[Any] = []
    configure(on_call_start=events.append)

    mock = MockProvider(['{"message": "hi", "word_count": 1}'])
    with _patch_provider(mock):
        await execute_prompt(
            model=MODEL,
            messages=[{"role": "user", "content": "hello"}],
            return_type=Greeting,
        )

    assert len(events) == 1
    assert isinstance(events[0], CallStartEvent)
    assert events[0].provider == "anthropic"
    assert events[0].model_id == "test-model"


@pytest.mark.asyncio
async def test_execute_prompt_fires_call_end_event() -> None:
    events: list[Any] = []
    configure(on_call_end=events.append)

    mock = MockProvider(['{"message": "hi", "word_count": 1}'])
    with _patch_provider(mock):
        await execute_prompt(
            model=MODEL,
            messages=[{"role": "user", "content": "hello"}],
            return_type=Greeting,
        )

    assert len(events) == 1
    assert isinstance(events[0], CallEndEvent)
    assert events[0].duration > 0
    assert events[0].provider == "anthropic"


@pytest.mark.asyncio
async def test_execute_prompt_fires_call_end_on_failure() -> None:
    events: list[Any] = []
    configure(on_call_end=events.append)

    mock = ErrorProvider(RuntimeError("unexpected"))
    with _patch_provider(mock):
        with pytest.raises(EamlProviderError):
            await execute_prompt(
                model=MODEL,
                messages=[{"role": "user", "content": "hello"}],
                return_type=Greeting,
            )

    assert len(events) == 1
    assert isinstance(events[0], CallEndEvent)
    assert events[0].provider == "anthropic"


@pytest.mark.asyncio
async def test_execute_prompt_wraps_unexpected_error() -> None:
    mock = ErrorProvider(RuntimeError("unexpected"))
    with _patch_provider(mock):
        with pytest.raises(EamlProviderError) as exc_info:
            await execute_prompt(
                model=MODEL,
                messages=[{"role": "user", "content": "hello"}],
                return_type=Greeting,
            )

    assert exc_info.value.provider == "anthropic"
    assert "unexpected" in str(exc_info.value)


@pytest.mark.asyncio
async def test_execute_prompt_passes_eaml_errors_through() -> None:
    mock = ErrorProvider(EamlConfigError("missing key"))
    with _patch_provider(mock):
        with pytest.raises(EamlConfigError, match="missing key"):
            await execute_prompt(
                model=MODEL,
                messages=[{"role": "user", "content": "hello"}],
                return_type=Greeting,
            )


@pytest.mark.asyncio
async def test_execute_prompt_passes_temperature() -> None:
    mock = MockProvider(['{"message": "hi", "word_count": 1}'])
    with _patch_provider(mock):
        await execute_prompt(
            model=MODEL,
            messages=[{"role": "user", "content": "hello"}],
            return_type=Greeting,
            temperature=0.7,
        )

    assert mock.calls[0]["temperature"] == 0.7


@pytest.mark.asyncio
async def test_execute_prompt_passes_max_tokens() -> None:
    mock = MockProvider(['{"message": "hi", "word_count": 1}'])
    with _patch_provider(mock):
        await execute_prompt(
            model=MODEL,
            messages=[{"role": "user", "content": "hello"}],
            return_type=Greeting,
            max_tokens=256,
        )

    assert mock.calls[0]["max_tokens"] == 256


@pytest.mark.asyncio
async def test_execute_prompt_default_max_retries() -> None:
    mock = MockProvider(["bad"] * 5)
    with _patch_provider(mock):
        with pytest.raises(Exception):  # noqa: B017
            await execute_prompt(
                model=MODEL,
                messages=[{"role": "user", "content": "hello"}],
                return_type=Greeting,
            )

    # Default max_retries=3, so provider called 3 times
    assert len(mock.calls) == 3


# --- Agent and ToolMetadata tests ---


def test_agent_subclass() -> None:
    class MyAgent(Agent):
        model = {"provider": "anthropic", "model_id": "test"}
        system_prompt = "Be helpful"
        max_turns = 5

    agent = MyAgent()
    assert agent.model == {"provider": "anthropic", "model_id": "test"}
    assert agent.system_prompt == "Be helpful"
    assert agent.max_turns == 5
    assert agent.on_error == "fail"


def test_agent_default_attributes() -> None:
    agent = Agent()
    assert agent.model == {}
    assert agent.tools == []
    assert agent.system_prompt == ""
    assert agent.max_turns == 10
    assert agent.on_error == "fail"
    assert agent.on_error_retries == 0


def test_tool_metadata_dataclass() -> None:
    def my_func() -> str:
        return "result"

    tool = ToolMetadata(
        name="search",
        description="Search the web",
        parameters=[{"name": "query", "type": "string"}],
        return_type="SearchResult",
        function=my_func,
    )

    assert tool.name == "search"
    assert tool.description == "Search the web"
    assert tool.parameters == [{"name": "query", "type": "string"}]
    assert tool.return_type == "SearchResult"
    assert tool.function is my_func
