"""Shared test fixtures for eaml_runtime tests."""

from __future__ import annotations

from typing import Any

import pytest

from eaml_runtime.providers import clear_provider_cache
from eaml_runtime.telemetry import _reset as reset_telemetry


@pytest.fixture()
def sample_anthropic_model() -> dict[str, Any]:
    return {
        "provider": "anthropic",
        "model_id": "anthropic/claude-3-haiku-20240307",
        "capabilities": [],
    }


@pytest.fixture()
def sample_openai_model() -> dict[str, Any]:
    return {
        "provider": "openai",
        "model_id": "openai/gpt-4o-mini",
        "capabilities": [],
    }


@pytest.fixture()
def sample_ollama_model() -> dict[str, Any]:
    return {
        "provider": "ollama",
        "model_id": "ollama/llama3",
        "capabilities": [],
    }


@pytest.fixture()
def sample_messages() -> list[dict[str, str]]:
    return [{"role": "user", "content": "Hello"}]


@pytest.fixture()
def sample_messages_with_system() -> list[dict[str, str]]:
    return [
        {"role": "system", "content": "Be helpful"},
        {"role": "user", "content": "Hello"},
    ]


@pytest.fixture(autouse=True)
def _cleanup_providers() -> Any:  # noqa: ANN401
    """Clear provider cache and telemetry hooks after each test."""
    yield
    clear_provider_cache()
    reset_telemetry()
