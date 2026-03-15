// ============================================================
// FILE: minimal.eaml
// PURPOSE: Minimal valid EAML program — smoke-test fixture
// EXPECTED: Clean compile, zero errors, zero warnings
// ============================================================

// A model with no capabilities — simplest valid model declaration.
model Haiku = Model(
  id: "anthropic/claude-3-haiku-20240307",
  provider: "anthropic",
  caps: []
)

// A minimal schema with two primitive-type fields.
schema Greeting {
  message: string
  word_count: int
}

// A minimal prompt — no requires clause, no optional fields.
// Returns the schema defined above with a single interpolated parameter.
prompt Greet(name: string) -> Greeting {
  user: "Say hello to {name} and count the words in your greeting."
}
