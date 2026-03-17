"""EAML Runtime -- runtime support for EAML-generated Python code."""

__version__ = "0.1.0"

from eaml_runtime.agent import Agent, ToolMetadata
from eaml_runtime.errors import (
    EamlConfigError,
    EamlError,
    EamlProviderError,
    EamlValidationError,
)
from eaml_runtime.telemetry import configure
from eaml_runtime.validation import execute_prompt

__all__ = [
    "Agent",
    "EamlConfigError",
    "EamlError",
    "EamlProviderError",
    "EamlValidationError",
    "ToolMetadata",
    "configure",
    "execute_prompt",
]
