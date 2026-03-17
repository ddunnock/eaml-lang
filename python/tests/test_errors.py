"""Tests for the EAML error hierarchy."""

from __future__ import annotations

from eaml_runtime.errors import (
    EamlConfigError,
    EamlError,
    EamlProviderError,
    EamlValidationError,
)


def test_eaml_error_is_exception() -> None:
    assert isinstance(EamlError("test"), Exception)


def test_config_error_is_eaml_error() -> None:
    assert isinstance(EamlConfigError("missing key"), EamlError)


def test_validation_error_is_eaml_error() -> None:
    err = EamlValidationError(
        model_id="test-model",
        attempts=3,
        last_response='{"bad": "json"}',
        errors=["field required"],
    )
    assert isinstance(err, EamlError)


def test_provider_error_is_eaml_error() -> None:
    err = EamlProviderError(provider="anthropic", message="rate limited", status_code=429)
    assert isinstance(err, EamlError)


def test_validation_error_attributes() -> None:
    err = EamlValidationError(
        model_id="claude-3",
        attempts=2,
        last_response="raw response",
        errors=["missing field 'name'", "invalid type"],
    )
    assert err.model_id == "claude-3"
    assert err.attempts == 2
    assert err.last_response == "raw response"
    assert err.errors == ["missing field 'name'", "invalid type"]


def test_validation_error_message_format() -> None:
    err = EamlValidationError(
        model_id="gpt-4o",
        attempts=3,
        last_response="{}",
        errors=["field required"],
    )
    msg = str(err)
    assert "Validation failed after 3 attempts" in msg
    assert "gpt-4o" in msg
    assert "field required" in msg


def test_provider_error_attributes() -> None:
    err = EamlProviderError(provider="openai", message="timeout", status_code=504)
    assert err.provider == "openai"
    assert err.status_code == 504


def test_provider_error_no_status_code() -> None:
    err = EamlProviderError(provider="ollama", message="connection refused")
    assert err.provider == "ollama"
    assert err.status_code is None


def test_provider_error_message_format() -> None:
    err = EamlProviderError(provider="anthropic", message="rate limited")
    msg = str(err)
    assert "Provider" in msg
    assert "anthropic" in msg
    assert "rate limited" in msg


def test_catch_eaml_error_catches_subtypes() -> None:
    """Catching EamlError should catch all subtype exceptions."""
    for exc in [
        EamlConfigError("config"),
        EamlValidationError("m", 1, "", []),
        EamlProviderError("p", "msg"),
    ]:
        try:
            raise exc
        except EamlError:
            pass  # Expected -- caught by base class
