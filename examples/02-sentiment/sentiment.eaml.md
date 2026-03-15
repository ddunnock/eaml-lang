// ============================================================
// FILE: sentiment.eaml
// PURPOSE: Sentiment analysis — canonical real-world reference
// EXPECTED: Clean compile, zero errors, zero warnings
// ============================================================

// A capable model with json_mode and streaming.
model Sonnet = Model(
  id: "anthropic/claude-3-5-sonnet-20241022",
  provider: "anthropic",
  caps: [json_mode, streaming]
)

// Structured output schema for sentiment classification.
schema SentimentResult {
  sentiment: "positive" | "neutral" | "negative"
  confidence: float<0.0, 1.0>
  explanation: string
}

// Sentiment analysis prompt with capability requirement.
// requires json_mode ensures structured JSON output.
prompt AnalyzeSentiment(text: string)
  requires json_mode
  -> SentimentResult
{
  system: "You are a sentiment analysis expert. Classify the sentiment of the given text and provide a confidence score between 0 and 1. Be concise in your explanation."
  user: "Analyze the sentiment of the following text:

{text}"
  temperature: 0.2
  max_tokens: 256
}
