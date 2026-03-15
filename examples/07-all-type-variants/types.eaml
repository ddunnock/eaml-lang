// ============================================================
// FILE: types.eaml
// PURPOSE: Complete type system exercise file
// EXPECTED: Clean compile, zero errors, zero warnings
// ============================================================

// A model declaration so prompts are valid.
model Claude = Model(
  id: "anthropic/claude-3-5-sonnet-20241022",
  provider: "anthropic",
  caps: [json_mode]
)

// --- Primitive Types ---

schema Primitives {
  label: string
  score: float
  count: int
  active: bool
  empty: null
}

// --- Bounded Types ---

schema BoundedTypes {
  probability: float<0.0, 1.0>
  rating: float<min: 0.0, max: 5.0>
  short_text: string<max: 200>
  description: string<min: 1, max: 1000>
  age: int<min: 0>
  percentage: int<min: 0, max: 100>
}

// --- Composite Types: Optional and Array ---

schema CompositeTypes {
  required_name: string
  optional_name: string?
  tag_list: int[]
  optional_tags: string[]?
  nullable_items: string?[]
  fully_optional: string?[]?
}

// --- Literal Union Types ---

schema UnionTypes {
  binary_choice: "yes" | "no"
  direction: "north" | "south" | "east" | "west"
  priority: "low" | "medium" | "high" | "critical"
}

// --- Schema as Field Type (Nominal Typing) ---

schema Address {
  street: string
  city: string
  zip_code: string<min: 1, max: 20>
}

schema Person {
  full_name: string
  email: string?
  address: Address
  previous_addresses: Address[]?
}

// --- Prompts Exercising Different Return Types ---

// Returns a schema type (TS-RET-01).
prompt DescribePerson(name: string)
  requires json_mode
  -> Person
{
  user: "Create a profile for a person named {name}."
}

// Returns a literal union type directly (TS-RET-03).
prompt ClassifyPriority(task: string)
  -> "low" | "medium" | "high" | "critical"
{
  user: "Classify the priority of this task: {task}"
}

// Returns a primitive type.
prompt Summarize(text: string) -> string {
  user: "Summarize the following in one sentence: {text}"
}
