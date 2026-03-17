"""LLM provider integrations."""

from __future__ import annotations

from abc import ABC, abstractmethod

from eaml_runtime.errors import EamlConfigError

__all__ = ["Provider", "get_provider", "clear_provider_cache"]


class Provider(ABC):
    """Abstract base class for LLM provider adapters."""

    @abstractmethod
    async def send_prompt(
        self,
        messages: list[dict[str, str]],
        model_id: str,
        *,
        temperature: float | None = None,
        max_tokens: int | None = None,
    ) -> str:
        """Send messages to the LLM and return the raw text response."""
        ...


_provider_cache: dict[str, Provider] = {}


def get_provider(provider_name: str) -> Provider:
    """Get or create a cached provider instance by name.

    Supported providers: "anthropic", "openai", "ollama".
    Raises EamlConfigError for unknown provider names.
    """
    if provider_name not in _provider_cache:
        if provider_name == "anthropic":
            from eaml_runtime.providers.anthropic import AnthropicProvider

            _provider_cache[provider_name] = AnthropicProvider()
        elif provider_name == "openai":
            from eaml_runtime.providers.openai import OpenAIProvider

            _provider_cache[provider_name] = OpenAIProvider()
        elif provider_name == "ollama":
            from eaml_runtime.providers.ollama import OllamaProvider

            _provider_cache[provider_name] = OllamaProvider()
        else:
            raise EamlConfigError(f"Unknown provider: {provider_name}")
    return _provider_cache[provider_name]


def clear_provider_cache() -> None:
    """Clear the provider cache. Used in tests."""
    _provider_cache.clear()
