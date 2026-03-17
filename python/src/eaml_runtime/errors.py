"""EAML runtime error hierarchy."""

from __future__ import annotations


class EamlError(Exception):
    """Base exception for all EAML runtime errors."""


class EamlConfigError(EamlError):
    """Missing API keys or invalid configuration."""


class EamlValidationError(EamlError):
    """LLM output failed Pydantic validation after all retries."""

    def __init__(
        self,
        model_id: str,
        attempts: int,
        last_response: str,
        errors: list[str],
    ) -> None:
        self.model_id = model_id
        self.attempts = attempts
        self.last_response = last_response
        self.errors = errors
        super().__init__(
            f"Validation failed after {attempts} attempts for model '{model_id}'. "
            f"Errors: {'; '.join(errors)}"
        )


class EamlProviderError(EamlError):
    """Provider API error (wrapped SDK exception)."""

    def __init__(
        self,
        provider: str,
        message: str,
        status_code: int | None = None,
    ) -> None:
        self.provider = provider
        self.status_code = status_code
        super().__init__(f"Provider '{provider}' error: {message}")
