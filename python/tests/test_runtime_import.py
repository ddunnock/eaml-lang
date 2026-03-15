"""Smoke test: verify the runtime package is importable."""


def test_runtime_imports() -> None:
    import eaml_runtime  # noqa: F811

    assert hasattr(eaml_runtime, "__version__")
