# Provider-Agnostic AI Features - Comprehensive Requirements

**Project**: PROJ-14
**Created**: 2025-12-09
**Plan Reference**: `/Users/clay/.claude/plans/happy-riding-pinwheel.md`

**Architectural Approach**: Provider-Agnostic (Option 1)
- Build features at abstraction layer for ALL providers (Claude, OpenAI, Gemini)
- Only 20% of features are provider-specific (grounding, safety settings)
- Users can switch providers without losing functionality

---

## PHASE 1: Essential Foundation (Sprint 1)

### REQ-214: System Instruction Support for All Providers

**Status**: PLANNED
**Priority**: CRITICAL
**Sprint**: 1

#### User Story
```
As a Radium developer,
I want system instructions to be properly preserved across all AI providers,
So that my prompts maintain their intended context and behavior regardless of which model I use.
```

#### Why This Matters

**Current Problem**: System messages are currently mapped to user messages for Gemini (lossy conversion), while Claude handles them natively. This creates inconsistent behavior across providers.

**Business Value**:
- Consistent prompt behavior across all providers
- Proper role separation (system vs user context)
- Better prompt engineering capabilities
- Foundation for multi-provider agent framework

**Technical Rationale**:
- Claude uses `system` parameter in API
- OpenAI uses `role: "system"` in messages array
- Gemini uses `systemInstruction` field (currently not implemented)
- Each provider has different system instruction handling

#### Acceptance Criteria

**Must Have**:
- [ ] Extract system messages from ChatMessage array for all providers
- [ ] Map system instructions to provider-specific format:
  - Claude: `system` parameter
  - OpenAI: `role: "system"` in messages
  - Gemini: `systemInstruction` field
- [ ] Remove system messages from user messages array
- [ ] Preserve backward compatibility for existing prompts
- [ ] Add integration tests for each provider

**Should Have**:
- [ ] Support multiple system messages (concatenate)
- [ ] Warning when system instruction exceeds provider limits

**Won't Have** (this phase):
- System instruction templates
- Dynamic system instruction injection

#### Provider Mapping

| Provider | Implementation | File Location |
|----------|----------------|---------------|
| Claude | Already supported | `radium-models/src/claude.rs:206` |
| OpenAI | Already supported | `radium-models/src/openai.rs` |
| Gemini | **NEW** - Add `systemInstruction` field | `radium-models/src/gemini.rs:115-121` |

#### Implementation Details

**Files to Modify**:
1. `/Users/clay/Development/RAD/crates/radium-models/src/gemini.rs`
   - Add `system_instruction: Option<GeminiSystemInstruction>` to `GeminiRequest`
   - Implement `extract_system_messages()` helper
   - Follow Claude's pattern (lines 206-213)

**New Structures**:
```rust
#[derive(Debug, Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiPart>,
}

fn extract_system_messages(messages: &[ChatMessage]) -> Option<GeminiSystemInstruction> {
    let system_content: String = messages
        .iter()
        .filter(|msg| msg.role == "system")
        .map(|msg| msg.content.clone())
        .collect::<Vec<_>>()
        .join("\n\n");

    if system_content.is_empty() {
        None
    } else {
        Some(GeminiSystemInstruction {
            parts: vec![GeminiPart::Text { text: system_content }],
        })
    }
}
```

#### Testing Strategy

**Unit Tests**:
- Test system message extraction
- Test multi-system message concatenation
- Test empty system messages (None)

**Integration Tests** (new file: `crates/radium-models/tests/gemini_system_test.rs`):
```rust
#[tokio::test]
async fn test_gemini_system_instruction() {
    let model = GeminiModel::new("gemini-2.0-flash-exp", api_key);
    let messages = vec![
        ChatMessage { role: "system", content: "You are a helpful assistant." },
        ChatMessage { role: "user", content: "Hello!" },
    ];

    let response = model.generate_chat_completion(&messages, None).await.unwrap();
    // Verify system instruction was sent in request
}
```

**E2E Tests**:
- CLI test with system message flag
- Verify consistent behavior across all 3 providers

#### Dependencies
- None (can implement immediately)

#### Risk Assessment
- **Low Risk**: Additive change, no breaking changes
- **Mitigation**: Extensive backward compatibility testing

---

### REQ-215: Extended Generation Parameters Support

**Status**: PLANNED
**Priority**: HIGH
**Sprint**: 1

#### User Story
```
As a Radium user,
I want full control over model generation parameters (top_k, frequency_penalty, response format),
So that I can fine-tune model outputs for my specific use case across all providers.
```

#### Why This Matters

**Current Problem**: Only basic parameters supported (temperature, top_p, max_tokens, stop_sequences). Missing:
- `top_k` (token sampling control)
- `frequency_penalty` and `presence_penalty` (repetition control)
- `response_format` (JSON mode, structured outputs)

**Business Value**:
- Better output quality through fine-tuning
- Structured outputs (JSON mode) for programmatic use
- Reduced repetition in generated text
- Competitive feature parity with native provider CLIs

**Technical Rationale**:
- All 3 providers support these parameters
- Foundation for REQ-225 (Structured Outputs)
- Enables advanced use cases (JSON responses, code generation)

#### Acceptance Criteria

**Must Have**:
- [ ] Add new fields to `ModelParameters` struct:
  - `top_k: Option<u32>`
  - `frequency_penalty: Option<f32>`
  - `presence_penalty: Option<f32>`
  - `response_format: Option<ResponseFormat>`
- [ ] Map parameters to provider-specific format:
  - Claude: `top_k`, `response_format`
  - OpenAI: `top_k`, `frequency_penalty`, `presence_penalty`, `response_format`
  - Gemini: `top_k`, `response_mime_type`, `response_schema`
- [ ] Add CLI flags for new parameters
- [ ] Add TOML config support for per-engine defaults
- [ ] Validate parameter ranges per provider

**Should Have**:
- [ ] Helpful error messages for invalid parameter combinations
- [ ] Provider capability detection (warn if unsupported)

#### Provider Mapping

| Parameter | Claude | OpenAI | Gemini | Notes |
|-----------|--------|--------|--------|-------|
| `top_k` | ✅ | ✅ | ✅ | All support |
| `frequency_penalty` | ❌ | ✅ | ❌ | OpenAI only |
| `presence_penalty` | ❌ | ✅ | ❌ | OpenAI only |
| `response_format` | ✅ (JSON) | ✅ (JSON) | ✅ (schema) | Gemini most flexible |

#### Implementation Details

**Files to Modify**:
1. `/Users/clay/Development/RAD/crates/radium-abstraction/src/lib.rs` (lines 69-84)
   ```rust
   pub struct ModelParameters {
       // Existing fields...
       pub top_k: Option<u32>,
       pub frequency_penalty: Option<f32>,    // Range: -2.0 to 2.0
       pub presence_penalty: Option<f32>,     // Range: -2.0 to 2.0
       pub response_format: Option<ResponseFormat>,
   }

   pub enum ResponseFormat {
       Text,
       Json,
       JsonSchema(String), // JSON schema string
   }
   ```

2. `/Users/clay/Development/RAD/crates/radium-models/src/gemini.rs` (lines 236-246)
   ```rust
   pub struct GeminiGenerationConfig {
       // Existing fields...
       pub top_k: Option<u32>,
       pub response_mime_type: Option<String>,  // "application/json"
       pub response_schema: Option<serde_json::Value>,
   }
   ```

3. `/Users/clay/Development/RAD/apps/cli/src/commands/step.rs`
   ```rust
   top_k: Option<u32>,
   frequency_penalty: Option<f32>,
   presence_penalty: Option<f32>,
   response_format: Option<String>,  // "json" | "text"
   ```

**Configuration Example**:
```toml
# ~/.radium/config.toml
[engines.gemini]
model = "gemini-2.0-flash-exp"
temperature = 0.7
top_k = 40
response_format = "json"

[engines.claude]
model = "claude-sonnet-4.5"
top_k = 50
```

#### Testing Strategy

**Unit Tests**:
- Parameter validation (range checks)
- Provider-specific mapping
- Default value handling

**Integration Tests**:
```rust
#[test]
fn test_json_response_format() {
    let params = ModelParameters {
        response_format: Some(ResponseFormat::Json),
        ..Default::default()
    };

    // Test with each provider
    test_gemini_json(&params);
    test_claude_json(&params);
    test_openai_json(&params);
}
```

**E2E Tests**:
- CLI with `--response-format json`
- Verify JSON output from all providers

#### Dependencies
- None

#### Risk Assessment
- **Low Risk**: All new fields are `Option<T>` (backward compatible)

---

### REQ-216: Response Metadata Capture for All Providers

**Status**: PLANNED
**Priority**: MEDIUM
**Sprint**: 1

#### User Story
```
As a Radium developer,
I want access to response metadata (finish_reason, safety_ratings, citations),
So that I can understand why a response was generated, handle safety blocks, and attribute sources.
```

#### Why This Matters

**Current Problem**: Only capturing content, model_id, and usage tokens. Missing critical metadata:
- `finish_reason` (stop, length, safety, tool_use)
- `safety_ratings` (Gemini safety categories)
- `citation_metadata` (grounding sources)
- `logprobs` (token probabilities for analysis)

**Business Value**:
- Better error handling (detect safety blocks vs length limits)
- Source attribution for grounded responses
- Analytics and debugging (finish reasons, safety patterns)
- Compliance (track when content is filtered)

**Technical Rationale**:
- Required for REQ-223 (Grounding) - need citations
- Required for REQ-224 (Safety Settings) - need ratings
- Foundation for analytics and monitoring

#### Acceptance Criteria

**Must Have**:
- [ ] Add `metadata: Option<HashMap<String, serde_json::Value>>` to `ModelResponse`
- [ ] Capture for all providers:
  - **finish_reason**: "stop" | "length" | "safety" | "tool_use" | "max_tokens"
  - **safety_ratings** (Gemini): Array of harm categories and scores
  - **citations** (Gemini grounding): Source URLs and snippets
  - **logprobs** (OpenAI): Token probabilities
- [ ] Add helper methods to `ModelResponse`:
  - `get_finish_reason() -> Option<String>`
  - `get_safety_ratings() -> Option<Vec<SafetyRating>>`
  - `get_citations() -> Option<Vec<Citation>>`
- [ ] CLI output formatting for metadata (optional `--show-metadata` flag)

**Should Have**:
- [ ] Pretty-print citations with source attribution
- [ ] Warning display for safety blocks

#### Provider Mapping

| Metadata | Claude | OpenAI | Gemini |
|----------|--------|--------|--------|
| finish_reason | ✅ `stop_reason` | ✅ `finish_reason` | ✅ `finish_reason` |
| safety_ratings | ❌ | ❌ | ✅ `safety_ratings` |
| citations | ❌ | ❌ | ✅ `citation_metadata` |
| logprobs | ❌ | ✅ `logprobs` | ❌ |

#### Implementation Details

**Files to Modify**:
1. `/Users/clay/Development/RAD/crates/radium-abstraction/src/lib.rs` (lines 98-108)
   ```rust
   pub struct ModelResponse {
       pub content: String,
       pub model_id: Option<String>,
       pub usage: Option<ModelUsage>,
       pub metadata: Option<HashMap<String, serde_json::Value>>, // NEW
   }

   impl ModelResponse {
       pub fn get_finish_reason(&self) -> Option<String> {
           self.metadata.as_ref()?
               .get("finish_reason")?
               .as_str()
               .map(String::from)
       }

       pub fn get_safety_ratings(&self) -> Option<Vec<SafetyRating>> {
           // Parse from metadata
       }
   }
   ```

2. `/Users/clay/Development/RAD/crates/radium-models/src/gemini.rs` (line 257)
   ```rust
   pub struct GeminiCandidate {
       pub content: GeminiContent,
       pub finish_reason: Option<String>,           // NEW
       pub safety_ratings: Option<Vec<GeminiSafetyRating>>, // NEW
       pub citation_metadata: Option<GeminiCitationMetadata>, // NEW
   }

   #[derive(Debug, Deserialize)]
   pub struct GeminiSafetyRating {
       pub category: String,  // "HARM_CATEGORY_HATE_SPEECH"
       pub probability: String, // "NEGLIGIBLE" | "LOW" | "MEDIUM" | "HIGH"
   }

   #[derive(Debug, Deserialize)]
   pub struct GeminiCitationMetadata {
       pub citations: Vec<GeminiCitation>,
   }

   #[derive(Debug, Deserialize)]
   pub struct GeminiCitation {
       pub start_index: u32,
       pub end_index: u32,
       pub uri: String,
       pub title: Option<String>,
   }
   ```

**Metadata Storage Example**:
```json
{
  "finish_reason": "stop",
  "safety_ratings": [
    {
      "category": "HARM_CATEGORY_HATE_SPEECH",
      "probability": "NEGLIGIBLE"
    }
  ],
  "citations": [
    {
      "start_index": 0,
      "end_index": 100,
      "uri": "https://example.com/source",
      "title": "Example Source"
    }
  ]
}
```

#### Testing Strategy

**Unit Tests**:
- Test metadata extraction from provider responses
- Test helper methods (get_finish_reason, etc.)
- Test missing metadata (graceful None handling)

**Integration Tests**:
```rust
#[tokio::test]
async fn test_gemini_metadata_capture() {
    let model = GeminiModel::new("gemini-2.0-flash-exp", api_key);
    let response = model.generate_text("Test prompt", None).await.unwrap();

    assert!(response.metadata.is_some());
    let finish_reason = response.get_finish_reason();
    assert!(finish_reason.is_some());
    assert!(["stop", "length", "safety"].contains(&finish_reason.unwrap().as_str()));
}
```

#### Dependencies
- None

#### Risk Assessment
- **Low Risk**: Additive change, metadata is optional

---

### REQ-217: Streaming Implementation for All Providers

**Status**: PLANNED
**Priority**: HIGH
**Sprint**: 1

#### User Story
```
As a Radium user,
I want to see model responses stream in real-time,
So that I get immediate feedback and can interrupt long generations.
```

#### Why This Matters

**Current Problem**: Only Claude has StreamingModel implementation. Gemini and OpenAI don't support streaming.

**Business Value**:
- Better UX (immediate feedback vs waiting)
- Ability to interrupt long/incorrect generations
- Perceived performance improvement
- Feature parity with provider CLIs

**Technical Rationale**:
- All providers support SSE (Server-Sent Events) streaming
- Foundation for interactive chat experiences
- Required for future TUI enhancements

#### Acceptance Criteria

**Must Have**:
- [ ] Implement `StreamingModel` trait for Gemini
- [ ] Implement `StreamingModel` trait for OpenAI
- [ ] Parse SSE events correctly for each provider
- [ ] Handle partial JSON chunks (reassemble multi-chunk responses)
- [ ] Add `--stream` flag to CLI `rad step` command
- [ ] Display streaming tokens in real-time to terminal
- [ ] Handle streaming errors (connection drops, rate limits)

**Should Have**:
- [ ] Progress indicator for streaming
- [ ] Graceful fallback to non-streaming on error
- [ ] Token rate display (tokens/sec)

#### Provider Mapping

| Provider | Streaming Endpoint | Event Format |
|----------|-------------------|--------------|
| Claude | `:streamMessages` | SSE with `event:` types |
| OpenAI | `stream=true` | SSE with `data:` JSON |
| Gemini | `:streamGenerateContent?alt=sse` | SSE with `data:` JSON |

#### Implementation Details

**Files to Modify**:
1. `/Users/clay/Development/RAD/crates/radium-models/src/gemini.rs`
   ```rust
   use futures::stream::{Stream, StreamExt};
   use std::pin::Pin;

   #[async_trait]
   impl StreamingModel for GeminiModel {
       async fn generate_stream(
           &self,
           messages: &[ChatMessage],
           params: Option<ModelParameters>,
       ) -> Result<Pin<Box<dyn Stream<Item = Result<String, ModelError>>>>, ModelError> {
           let url = format!(
               "{}/models/{}:streamGenerateContent?alt=sse&key={}",
               self.base_url, self.model_id, self.api_key
           );

           let response = self.client.post(&url)
               .json(&request)
               .send()
               .await?;

           let stream = response.bytes_stream()
               .map(|chunk| {
                   // Parse SSE: "data: {...}\n\n"
                   // Extract content from JSON
               });

           Ok(Box::pin(stream))
       }
   }
   ```

2. `/Users/clay/Development/RAD/apps/cli/src/commands/step.rs`
   ```rust
   stream: bool,  // --stream flag

   // In execute():
   if stream {
       let mut stream = engine.generate_stream(&messages, params).await?;
       while let Some(chunk) = stream.next().await {
           print!("{}", chunk?);  // Real-time display
           stdout().flush()?;
       }
   } else {
       // Existing non-streaming path
   }
   ```

**SSE Parsing Logic**:
```rust
fn parse_sse_chunk(chunk: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(chunk);
    for line in text.lines() {
        if line.starts_with("data: ") {
            let json_str = &line[6..]; // Skip "data: "
            if let Ok(data) = serde_json::from_str::<GeminiStreamResponse>(json_str) {
                return data.candidates
                    .first()?
                    .content
                    .parts
                    .first()?
                    .text
                    .clone();
            }
        }
    }
    None
}
```

#### Testing Strategy

**Unit Tests**:
- Test SSE parsing with sample chunks
- Test multi-chunk reassembly
- Test error handling (invalid JSON)

**Integration Tests**:
```rust
#[tokio::test]
async fn test_gemini_streaming() {
    let model = GeminiModel::new("gemini-2.0-flash-exp", api_key);
    let messages = vec![
        ChatMessage { role: "user", content: "Count to 10" },
    ];

    let mut stream = model.generate_stream(&messages, None).await.unwrap();
    let mut chunks = Vec::new();

    while let Some(chunk) = stream.next().await {
        chunks.push(chunk.unwrap());
    }

    assert!(!chunks.is_empty());
    let full_response = chunks.join("");
    assert!(full_response.contains("10"));
}
```

**E2E Tests**:
- CLI test with `--stream` flag
- Verify real-time output (not buffered)
- Test interrupt (Ctrl+C)

#### Dependencies
- None (can implement immediately)

#### Risk Assessment
- **Medium Risk**: SSE parsing can be fragile
- **Mitigation**: Extensive error handling, fallback to non-streaming

---

## PHASE 2: Multimodal Core (Sprints 2-4)

### REQ-218: Message Structure Redesign for Multimodal Content

**Status**: PLANNED
**Priority**: CRITICAL (BREAKING CHANGE)
**Sprint**: 2

#### User Story
```
As a Radium developer,
I want to send images, audio, video, and files alongside text prompts,
So that I can build multimodal applications that analyze media content.
```

#### Why This Matters

**Current Problem**: `ChatMessage` only supports text content (`content: String`). Cannot send:
- Images (screenshots, diagrams)
- Audio (transcription, analysis)
- Video (scene understanding)
- PDFs (document analysis)

**Business Value**:
- Unlock multimodal use cases (vision, audio, document analysis)
- Competitive feature parity (Claude, GPT-4o, Gemini all support multimodal)
- Foundation for future features (video understanding, OCR)
- Market differentiation (multi-provider multimodal)

**Technical Rationale**:
- **BREAKING CHANGE**: Must redesign message structure
- Foundation for REQ-219, REQ-220, REQ-221
- Must maintain backward compatibility via `From<String>`

#### Acceptance Criteria

**Must Have**:
- [ ] Replace `content: String` with `content: MessageContent` in `ChatMessage`
- [ ] Implement `MessageContent` enum:
  - `Text(String)` - backward compatible
  - `Blocks(Vec<ContentBlock>)` - multimodal
- [ ] Implement `ContentBlock` enum:
  - `Text { text: String }`
  - `Image { source: ImageSource }`
  - `Audio { source: MediaSource }`
  - `Video { source: MediaSource }`
  - `File { source: FileSource }`
- [ ] Implement `From<String> for MessageContent` (backward compat)
- [ ] Update all model implementations to handle new structure
- [ ] Migration guide for existing code

**Should Have**:
- [ ] Helper methods: `ChatMessage::text()`, `ChatMessage::with_image()`
- [ ] Validation (file sizes, mime types)

#### Provider Mapping

All 3 providers support multimodal content with similar structures:

| Provider | Text | Image | Audio | Video | PDF |
|----------|------|-------|-------|-------|-----|
| Claude | ✅ | ✅ base64 | ❌ | ❌ | ✅ base64 |
| OpenAI | ✅ | ✅ URL/base64 | ✅ base64 | ❌ | ❌ |
| Gemini | ✅ | ✅ inline/file | ✅ inline/file | ✅ inline/file | ✅ inline/file |

#### Implementation Details

**Files to Modify**:
1. `/Users/clay/Development/RAD/crates/radium-abstraction/src/lib.rs` (line 65)
   ```rust
   pub struct ChatMessage {
       pub role: String,
       pub content: MessageContent,  // CHANGED from String
   }

   pub enum MessageContent {
       Text(String),              // Backward compatible
       Blocks(Vec<ContentBlock>), // New multimodal
   }

   pub enum ContentBlock {
       Text { text: String },
       Image { source: ImageSource },
       Audio { source: MediaSource },
       Video { source: MediaSource },
       File { source: FileSource },
   }

   pub enum ImageSource {
       Url(String),
       Base64 { data: String, mime_type: String },
       Path(PathBuf),
   }

   pub enum MediaSource {
       Url(String),
       Base64 { data: String, mime_type: String },
       Path(PathBuf),
       FileApi { file_id: String }, // For uploaded files
   }

   pub enum FileSource {
       Path(PathBuf),
       Base64 { data: String, mime_type: String },
       FileApi { file_id: String },
   }

   // Backward compatibility
   impl From<String> for MessageContent {
       fn from(s: String) -> Self {
           MessageContent::Text(s)
       }
   }

   impl ChatMessage {
       pub fn text(role: impl Into<String>, content: impl Into<String>) -> Self {
           Self {
               role: role.into(),
               content: MessageContent::Text(content.into()),
           }
       }

       pub fn with_image(
           role: impl Into<String>,
           text: impl Into<String>,
           image_path: PathBuf,
       ) -> Self {
           Self {
               role: role.into(),
               content: MessageContent::Blocks(vec![
                   ContentBlock::Text { text: text.into() },
                   ContentBlock::Image {
                       source: ImageSource::Path(image_path),
                   },
               ]),
           }
       }
   }
   ```

2. Update ALL model implementations:
   - `/Users/clay/Development/RAD/crates/radium-models/src/claude.rs`
   - `/Users/clay/Development/RAD/crates/radium-models/src/openai.rs`
   - `/Users/clay/Development/RAD/crates/radium-models/src/gemini.rs`

**Migration Example**:
```rust
// OLD (still works)
let msg = ChatMessage {
    role: "user".to_string(),
    content: "Hello".to_string(),
};

// NEW (explicit)
let msg = ChatMessage {
    role: "user".to_string(),
    content: MessageContent::Text("Hello".to_string()),
};

// NEW (helper)
let msg = ChatMessage::text("user", "Hello");

// MULTIMODAL (new capability)
let msg = ChatMessage::with_image(
    "user",
    "What's in this image?",
    PathBuf::from("photo.jpg"),
);
```

#### Testing Strategy

**Unit Tests**:
- Test backward compatibility (`From<String>`)
- Test all content block types
- Test helper methods

**Integration Tests**:
```rust
#[test]
fn test_backward_compatibility() {
    // Old code should still compile
    let msg = ChatMessage {
        role: "user".to_string(),
        content: "Hello".into(), // From<String> automatic
    };

    match msg.content {
        MessageContent::Text(text) => assert_eq!(text, "Hello"),
        _ => panic!("Expected Text variant"),
    }
}

#[test]
fn test_multimodal_blocks() {
    let msg = ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Blocks(vec![
            ContentBlock::Text { text: "Analyze this".to_string() },
            ContentBlock::Image {
                source: ImageSource::Path(PathBuf::from("test.jpg")),
            },
        ]),
    };

    match msg.content {
        MessageContent::Blocks(blocks) => assert_eq!(blocks.len(), 2),
        _ => panic!("Expected Blocks variant"),
    }
}
```

**Regression Tests**:
- Run all existing tests to ensure backward compatibility
- Verify no breaking changes in existing code

#### Dependencies
- **MUST COMPLETE BEFORE**: REQ-219, REQ-220, REQ-221

#### Risk Assessment
- **HIGH RISK**: Breaking change to core abstraction
- **Mitigation**:
  - `From<String>` implementation for seamless migration
  - Extensive backward compatibility testing
  - Gradual rollout with feature flag

---

### REQ-219: Multimodal Content Support (Images, Audio, Video, PDFs)

**Status**: PLANNED
**Priority**: HIGH
**Sprint**: 3

#### User Story
```
As a Radium user,
I want to analyze images, transcribe audio, understand videos, and extract text from PDFs,
So that I can build applications that work with multimedia content.
```

#### Why This Matters

**Current Problem**: Radium only supports text inputs. Cannot:
- Analyze images (screenshots, diagrams, photos)
- Transcribe audio (meetings, calls)
- Understand videos (scenes, actions)
- Extract text from PDFs (documents, reports)

**Business Value**:
- Unlock vision use cases (OCR, object detection, scene understanding)
- Unlock audio use cases (transcription, sentiment analysis)
- Unlock document use cases (PDF parsing, form extraction)
- Competitive parity with Claude Desktop, ChatGPT

**Technical Rationale**:
- All 3 providers support multimodal (Gemini most comprehensive)
- Foundation for advanced workflows (document processing, media analysis)
- Requires REQ-218 (Message Structure Redesign)

#### Acceptance Criteria

**Must Have**:
- [ ] Implement multimodal support for all providers:
  - **Images**: PNG, JPEG, WebP, GIF
  - **Audio**: MP3, WAV, AAC, FLAC, OGG
  - **Video**: MP4, MOV, AVI (Gemini only)
  - **PDF**: Document analysis (Claude, Gemini)
- [ ] Support base64 encoding for small files (<20MB)
- [ ] Automatic mime type detection
- [ ] Size limit validation per provider
- [ ] Convert `ImageSource`/`MediaSource` to provider-specific format

**Should Have**:
- [ ] Automatic image compression for large files
- [ ] Cache loaded files (avoid re-reading)
- [ ] Progress indicator for large file processing

#### Provider Mapping

| Media Type | Claude | OpenAI | Gemini | Max Size |
|------------|--------|--------|--------|----------|
| Images (PNG, JPEG, WebP) | ✅ base64 | ✅ URL/base64 | ✅ inline/file | Claude: 5MB, Gemini: 20MB |
| Audio (MP3, WAV) | ❌ | ✅ base64 | ✅ inline/file | OpenAI: 25MB, Gemini: 20MB |
| Video (MP4, MOV) | ❌ | ❌ | ✅ inline/file | Gemini: 20MB inline, 2GB file API |
| PDF | ✅ base64 | ❌ | ✅ inline/file | Claude: 5MB, Gemini: 20MB |

#### Implementation Details

**Files to Modify**:
1. `/Users/clay/Development/RAD/crates/radium-models/src/gemini.rs`
   ```rust
   #[derive(Debug, Serialize)]
   #[serde(untagged)]
   enum GeminiPart {
       Text { text: String },
       InlineData { inline_data: GeminiInlineData },
       FileData { file_data: GeminiFileData },
   }

   #[derive(Debug, Serialize)]
   struct GeminiInlineData {
       mime_type: String,
       data: String, // base64
   }

   #[derive(Debug, Serialize)]
   struct GeminiFileData {
       mime_type: String,
       file_uri: String, // gs://bucket/file or File API URI
   }

   fn convert_content_blocks_to_parts(
       blocks: &[ContentBlock],
   ) -> Result<Vec<GeminiPart>, ModelError> {
       blocks.iter().map(|block| {
           match block {
               ContentBlock::Text { text } => {
                   Ok(GeminiPart::Text { text: text.clone() })
               }
               ContentBlock::Image { source } => {
                   let (mime_type, data) = match source {
                       ImageSource::Path(path) => {
                           let bytes = std::fs::read(path)?;
                           let mime = detect_mime_type(path)?;
                           let base64 = base64::encode(&bytes);
                           (mime, base64)
                       }
                       ImageSource::Base64 { data, mime_type } => {
                           (mime_type.clone(), data.clone())
                       }
                       ImageSource::Url(_) => {
                           // Fetch and convert to base64
                       }
                   };

                   Ok(GeminiPart::InlineData {
                       inline_data: GeminiInlineData { mime_type, data },
                   })
               }
               // Similar for Audio, Video, File...
           }
       }).collect()
   }
   ```

2. `/Users/clay/Development/RAD/crates/radium-models/src/claude.rs`
   ```rust
   // Claude uses similar structure with base64 encoding
   #[derive(Debug, Serialize)]
   #[serde(tag = "type")]
   enum ClaudeContentBlock {
       #[serde(rename = "text")]
       Text { text: String },

       #[serde(rename = "image")]
       Image {
           source: ClaudeImageSource,
       },
   }

   #[derive(Debug, Serialize)]
   struct ClaudeImageSource {
       #[serde(rename = "type")]
       source_type: String, // "base64"
       media_type: String,  // "image/jpeg"
       data: String,        // base64
   }
   ```

**Mime Type Detection**:
```rust
fn detect_mime_type(path: &Path) -> Result<String, ModelError> {
    match path.extension().and_then(|s| s.to_str()) {
        Some("png") => Ok("image/png".to_string()),
        Some("jpg") | Some("jpeg") => Ok("image/jpeg".to_string()),
        Some("webp") => Ok("image/webp".to_string()),
        Some("mp3") => Ok("audio/mp3".to_string()),
        Some("wav") => Ok("audio/wav".to_string()),
        Some("mp4") => Ok("video/mp4".to_string()),
        Some("pdf") => Ok("application/pdf".to_string()),
        _ => Err(ModelError::UnsupportedMediaType(
            path.display().to_string()
        )),
    }
}
```

#### Testing Strategy

**Unit Tests**:
- Test mime type detection
- Test base64 encoding
- Test size validation
- Test each content block type

**Integration Tests** (with real API calls):
```rust
#[tokio::test]
async fn test_gemini_image_analysis() {
    let model = GeminiModel::new("gemini-2.0-flash-exp", api_key);
    let msg = ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Blocks(vec![
            ContentBlock::Text {
                text: "What's in this image?".to_string(),
            },
            ContentBlock::Image {
                source: ImageSource::Path(PathBuf::from("test.jpg")),
            },
        ]),
    };

    let response = model.generate_chat_completion(&[msg], None).await.unwrap();
    assert!(!response.content.is_empty());
}

#[tokio::test]
async fn test_claude_pdf_analysis() {
    // Similar test for Claude PDF support
}
```

**E2E Tests**:
- Test each media type with CLI
- Test error handling (unsupported formats, size limits)

#### Dependencies
- **REQUIRES**: REQ-218 (Message Structure Redesign) MUST be complete

#### Risk Assessment
- **Medium Risk**: File I/O, base64 encoding can be slow
- **Mitigation**:
  - Async file reading
  - Caching
  - Progress indicators

---

### REQ-220: File API Integration for Large Media

**Status**: PLANNED
**Priority**: MEDIUM
**Sprint**: 3

#### User Story
```
As a Radium user working with large media files,
I want files over 20MB to automatically upload via File API,
So that I can analyze large videos and documents without manual uploads.
```

#### Why This Matters

**Current Problem**: Inline base64 encoding has size limits:
- Claude: 5MB
- Gemini: 20MB inline, but 2GB via File API
- OpenAI: 25MB

**Business Value**:
- Support large files (videos, high-res images, long audio)
- Automatic optimization (no manual file management)
- Better user experience (transparent file handling)

**Technical Rationale**:
- Gemini File API allows up to 2GB files
- Automatic upload when size > 20MB
- File cleanup and lifecycle management

#### Acceptance Criteria

**Must Have**:
- [ ] Implement Gemini File API client:
  - `upload_file(path: &Path) -> Result<GeminiFile>`
  - `delete_file(name: &str) -> Result<()>`
  - `list_files() -> Result<Vec<GeminiFile>>`
  - `get_file(name: &str) -> Result<GeminiFile>`
- [ ] Automatic upload logic:
  - If file size > 20MB, use File API
  - If file size ≤ 20MB, use inline base64
- [ ] File lifecycle management:
  - Upload before request
  - Automatic cleanup after response
  - Optional keep-alive for repeated use
- [ ] CLI commands for manual file management

**Should Have**:
- [ ] Progress indicator for large uploads
- [ ] Retry logic for failed uploads
- [ ] File caching (avoid re-upload)

#### Provider Mapping

| Provider | File API | Max Size | Upload Method |
|----------|----------|----------|---------------|
| Claude | ❌ | 5MB inline | base64 only |
| OpenAI | ❌ | 25MB inline | base64 only |
| Gemini | ✅ | 2GB | Multipart upload |

**Note**: Only Gemini supports File API. For Claude/OpenAI, reject files > limits.

#### Implementation Details

**New File**:
`/Users/clay/Development/RAD/crates/radium-models/src/gemini/file_api.rs`

```rust
use reqwest::multipart::{Form, Part};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct GeminiFile {
    pub name: String,          // "files/abc123"
    pub display_name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub create_time: String,
    pub expiration_time: String,
    pub sha256_hash: String,
    pub uri: String,
    pub state: String,         // "PROCESSING" | "ACTIVE" | "FAILED"
}

pub struct GeminiFileApi {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl GeminiFileApi {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://generativelanguage.googleapis.com/upload/v1beta".to_string(),
        }
    }

    pub async fn upload_file(&self, file_path: &Path) -> Result<GeminiFile, ModelError> {
        let file_bytes = tokio::fs::read(file_path).await?;
        let file_name = file_path.file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ModelError::InvalidInput("Invalid filename".into()))?;
        let mime_type = detect_mime_type(file_path)?;

        let form = Form::new()
            .part("file", Part::bytes(file_bytes)
                .file_name(file_name.to_string())
                .mime_str(&mime_type)?);

        let url = format!("{}/files?key={}", self.base_url, self.api_key);
        let response = self.client.post(&url)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ModelError::ApiError(response.text().await?));
        }

        let mut file: GeminiFile = response.json().await?;

        // Wait for file to be processed
        while file.state == "PROCESSING" {
            tokio::time::sleep(Duration::from_secs(1)).await;
            file = self.get_file(&file.name).await?;
        }

        if file.state == "FAILED" {
            return Err(ModelError::FileProcessingError(file.name));
        }

        Ok(file)
    }

    pub async fn delete_file(&self, name: &str) -> Result<(), ModelError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/{}?key={}",
            name, self.api_key
        );
        self.client.delete(&url).send().await?;
        Ok(())
    }

    pub async fn list_files(&self) -> Result<Vec<GeminiFile>, ModelError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/files?key={}",
            self.api_key
        );
        let response: ListFilesResponse = self.client.get(&url).send().await?.json().await?;
        Ok(response.files)
    }

    pub async fn get_file(&self, name: &str) -> Result<GeminiFile, ModelError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/{}?key={}",
            name, self.api_key
        );
        let file = self.client.get(&url).send().await?.json().await?;
        Ok(file)
    }
}

#[derive(Debug, Deserialize)]
struct ListFilesResponse {
    files: Vec<GeminiFile>,
}
```

**Integration with GeminiModel**:
```rust
impl GeminiModel {
    async fn handle_large_file(&self, source: &MediaSource) -> Result<GeminiPart, ModelError> {
        match source {
            MediaSource::Path(path) => {
                let metadata = std::fs::metadata(path)?;

                if metadata.len() > 20 * 1024 * 1024 { // 20MB
                    // Use File API
                    let file_api = GeminiFileApi::new(self.api_key.clone());
                    let file = file_api.upload_file(path).await?;

                    Ok(GeminiPart::FileData {
                        file_data: GeminiFileData {
                            mime_type: file.mime_type,
                            file_uri: file.uri,
                        },
                    })
                } else {
                    // Use inline base64
                    let bytes = std::fs::read(path)?;
                    let mime = detect_mime_type(path)?;
                    let data = base64::encode(&bytes);

                    Ok(GeminiPart::InlineData {
                        inline_data: GeminiInlineData { mime_type: mime, data },
                    })
                }
            }
            MediaSource::FileApi { file_id } => {
                // Already uploaded
                Ok(GeminiPart::FileData {
                    file_data: GeminiFileData {
                        mime_type: "".to_string(), // Fetch from API
                        file_uri: file_id.clone(),
                    },
                })
            }
            // ... other variants
        }
    }
}
```

**CLI Commands**:
```bash
# New commands in rad models
rad models file-upload <path>
rad models file-list
rad models file-delete <file-id>
rad models file-info <file-id>

# Auto-upload flag
rad step analyze "Summarize video" --file large-video.mp4 --auto-upload
```

#### Testing Strategy

**Unit Tests**:
- Test size threshold logic (20MB)
- Test mime type detection
- Test file state polling

**Integration Tests** (requires Gemini API key):
```rust
#[tokio::test]
async fn test_file_api_upload() {
    let api_key = env::var("GEMINI_API_KEY").unwrap();
    let file_api = GeminiFileApi::new(api_key);

    // Create test file > 20MB
    let test_file = create_large_test_file(25 * 1024 * 1024);

    let file = file_api.upload_file(&test_file).await.unwrap();
    assert_eq!(file.state, "ACTIVE");

    // Cleanup
    file_api.delete_file(&file.name).await.unwrap();
}
```

**E2E Tests**:
- Test automatic upload with large file
- Test file cleanup after use
- Test error handling (upload failure)

#### Dependencies
- **REQUIRES**: REQ-218, REQ-219

#### Risk Assessment
- **Medium Risk**: File uploads can fail, need robust error handling
- **Mitigation**:
  - Retry logic
  - State polling
  - Automatic cleanup

---

### REQ-221: CLI Multimodal Input Handling

**Status**: PLANNED
**Priority**: HIGH
**Sprint**: 4

#### User Story
```
As a Radium CLI user,
I want convenient flags to attach images, audio, and files to my prompts,
So that I can quickly analyze multimedia content from the command line.
```

#### Why This Matters

**Current Problem**: CLI only accepts text prompts. No way to:
- Attach images for vision tasks
- Attach audio for transcription
- Attach files for document analysis
- Batch process multiple media files

**Business Value**:
- Convenient multimodal workflows
- Productivity boost (no manual base64 encoding)
- Feature parity with provider CLIs
- Foundation for automation scripts

**Technical Rationale**:
- Natural CLI UX (like `--file` in curl)
- Leverages REQ-218, REQ-219, REQ-220
- Enables batch processing use cases

#### Acceptance Criteria

**Must Have**:
- [ ] Add CLI flags to `rad step`:
  - `--image <path>` (multiple allowed)
  - `--audio <path>` (multiple allowed)
  - `--video <path>` (multiple allowed)
  - `--file <path>` (multiple allowed, for PDFs)
  - `--auto-upload` (use File API for large files)
- [ ] Construct `MessageContent::Blocks` from flags
- [ ] Support multiple media attachments in single prompt
- [ ] Helpful error messages (unsupported formats, missing files)
- [ ] Display media metadata in output (filename, size, mime type)

**Should Have**:
- [ ] Batch mode: `--batch <file>` with list of prompts + media
- [ ] Preview mode: show what will be sent before API call
- [ ] Support stdin for images (piping)

#### CLI Usage Examples

```bash
# Image analysis
rad step analyze "What's in this screenshot?" --image screenshot.png

# Multiple images
rad step compare "Compare these images" --image photo1.jpg --image photo2.jpg

# Audio transcription
rad step transcribe "Transcribe this meeting" --audio meeting.mp3

# Video analysis (Gemini only)
rad step analyze "Describe what happens" --video clip.mp4

# PDF document analysis
rad step summarize "Summarize this report" --file report.pdf

# Mixed content
rad step analyze "Analyze this presentation" \
  --file slides.pdf \
  --image diagram.png \
  --audio narration.mp3

# Large file auto-upload
rad step analyze "Summarize this long video" \
  --video large.mp4 \
  --auto-upload

# Batch processing
rad step analyze --batch prompts.txt
# prompts.txt format:
# prompt1 | image1.jpg
# prompt2 | audio1.mp3
# prompt3 | file1.pdf,image2.png
```

#### Implementation Details

**Files to Modify**:
`/Users/clay/Development/RAD/apps/cli/src/commands/step.rs`

```rust
pub async fn execute(
    id: String,
    prompt: Vec<String>,
    agent_config: Option<PathBuf>,
    engine_config: Option<PathBuf>,
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: bool,
    // NEW FLAGS
    image: Vec<PathBuf>,
    audio: Vec<PathBuf>,
    video: Vec<PathBuf>,
    file: Vec<PathBuf>,
    auto_upload: bool,
    batch: Option<PathBuf>,
) -> anyhow::Result<()> {
    // Construct content blocks
    let mut blocks = vec![
        ContentBlock::Text {
            text: prompt.join(" "),
        },
    ];

    // Add images
    for img in image {
        validate_file_exists(&img)?;
        blocks.push(ContentBlock::Image {
            source: if auto_upload {
                // Check size and decide
                let metadata = std::fs::metadata(&img)?;
                if metadata.len() > 20 * 1024 * 1024 {
                    ImageSource::Path(img) // Will trigger File API upload
                } else {
                    ImageSource::Path(img) // Will use inline base64
                }
            } else {
                ImageSource::Path(img)
            },
        });
    }

    // Add audio
    for aud in audio {
        validate_file_exists(&aud)?;
        blocks.push(ContentBlock::Audio {
            source: MediaSource::Path(aud),
        });
    }

    // Add videos
    for vid in video {
        validate_file_exists(&vid)?;
        blocks.push(ContentBlock::Video {
            source: MediaSource::Path(vid),
        });
    }

    // Add files (PDFs)
    for f in file {
        validate_file_exists(&f)?;
        blocks.push(ContentBlock::File {
            source: FileSource::Path(f),
        });
    }

    let message = ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Blocks(blocks),
    };

    // Display media metadata
    println!("📎 Attachments:");
    for (i, img) in image.iter().enumerate() {
        let size = std::fs::metadata(img)?.len();
        println!("  {} {}: {} ({} bytes)",
            if auto_upload && size > 20_000_000 { "☁️" } else { "📷" },
            i + 1,
            img.display(),
            size
        );
    }

    // ... rest of execution
}

fn validate_file_exists(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }
    Ok(())
}
```

**Clap Args**:
```rust
#[derive(Parser, Debug)]
pub struct StepArgs {
    // Existing args...

    /// Image files to attach (can specify multiple)
    #[arg(long, value_name = "PATH")]
    image: Vec<PathBuf>,

    /// Audio files to attach (can specify multiple)
    #[arg(long, value_name = "PATH")]
    audio: Vec<PathBuf>,

    /// Video files to attach (can specify multiple)
    #[arg(long, value_name = "PATH")]
    video: Vec<PathBuf>,

    /// Files to attach (PDFs, documents)
    #[arg(long, value_name = "PATH")]
    file: Vec<PathBuf>,

    /// Automatically upload large files via File API
    #[arg(long)]
    auto_upload: bool,

    /// Batch process from file
    #[arg(long, value_name = "PATH")]
    batch: Option<PathBuf>,
}
```

#### Testing Strategy

**Unit Tests**:
- Test content block construction from flags
- Test file validation
- Test batch file parsing

**Integration Tests**:
```rust
#[test]
fn test_cli_image_flag() {
    let output = Command::new("rad")
        .args(&[
            "step",
            "analyze",
            "What's in this image?",
            "--image",
            "test.jpg",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}
```

**E2E Tests**:
- Test each flag type
- Test multiple attachments
- Test auto-upload with large file
- Test batch mode

#### Dependencies
- **REQUIRES**: REQ-218, REQ-219, REQ-220

#### Risk Assessment
- **Low Risk**: Straightforward CLI argument parsing
- **Mitigation**: Validation and helpful error messages

---

## PHASE 3: Advanced Features (Sprint 5)

### REQ-222: Function Calling Enhancements for All Providers

**Status**: PLANNED
**Priority**: MEDIUM
**Sprint**: 5

#### User Story
```
As a Radium developer building agentic workflows,
I want improved function calling with parallel execution and AUTO/ANY/NONE modes,
So that models can reliably use tools across all providers.
```

#### Why This Matters

**Current Problem**: Basic orchestration layer exists, but missing:
- Parallel function calls
- Tool use modes (AUTO, ANY, NONE)
- Tool choice forcing
- Better error handling

**Business Value**:
- Unlock agentic workflows (models calling tools autonomously)
- Better tool use reliability
- Competitive parity with provider SDKs

**Technical Rationale**:
- All 3 providers support function calling
- Orchestration layer needs enhancement
- Foundation for complex agent workflows

#### Acceptance Criteria

**Must Have**:
- [ ] Add `tool_calls: Option<Vec<ToolCall>>` to `ModelResponse`
- [ ] Add `tool_config: Option<ToolConfig>` to request parameters
- [ ] Implement tool use modes:
  - `AUTO`: Model decides when to use tools
  - `ANY`: Force at least one tool call
  - `NONE`: Disable tool use
- [ ] Support parallel function calls (multiple tools in one turn)
- [ ] Add tool call tracking in response metadata
- [ ] Better error handling for tool execution failures

**Should Have**:
- [ ] Tool choice forcing (`required_tool: Some("specific_tool")`)
- [ ] Tool call retry logic
- [ ] Tool call logging/analytics

#### Provider Mapping

| Feature | Claude | OpenAI | Gemini |
|---------|--------|--------|--------|
| Function calling | ✅ `tools` | ✅ `tools` | ✅ `tools` |
| Tool modes | ✅ `tool_choice` | ✅ `tool_choice` | ✅ `function_calling_config.mode` |
| Parallel calls | ✅ | ✅ | ✅ |
| Tool forcing | ✅ | ✅ | ✅ |

#### Implementation Details

**Files to Modify**:
1. `/Users/clay/Development/RAD/crates/radium-abstraction/src/lib.rs`
   ```rust
   pub struct ModelResponse {
       pub content: String,
       pub model_id: Option<String>,
       pub usage: Option<ModelUsage>,
       pub metadata: Option<HashMap<String, serde_json::Value>>,
       pub tool_calls: Option<Vec<ToolCall>>, // NEW
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ToolCall {
       pub id: String,
       pub name: String,
       pub arguments: serde_json::Value,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ToolConfig {
       pub mode: ToolUseMode,
       pub required_tool: Option<String>,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub enum ToolUseMode {
       Auto,    // Model decides
       Any,     // Force at least one tool
       None,    // Disable tools
   }
   ```

2. `/Users/clay/Development/RAD/crates/radium-orchestrator/src/orchestration/providers/gemini.rs`
   ```rust
   impl GeminiOrchestrationProvider {
       fn build_tool_config(&self, config: &ToolConfig) -> GeminiFunctionCallingConfig {
           GeminiFunctionCallingConfig {
               mode: match config.mode {
                   ToolUseMode::Auto => "AUTO".to_string(),
                   ToolUseMode::Any => "ANY".to_string(),
                   ToolUseMode::None => "NONE".to_string(),
               },
               allowed_function_names: config.required_tool.as_ref()
                   .map(|t| vec![t.clone()]),
           }
       }
   }
   ```

3. `/Users/clay/Development/RAD/crates/radium-models/src/gemini.rs`
   ```rust
   #[derive(Debug, Serialize)]
   struct GeminiRequest {
       contents: Vec<GeminiContent>,
       generation_config: Option<GeminiGenerationConfig>,
       tools: Option<Vec<GeminiTool>>,
       tool_config: Option<GeminiToolConfig>, // NEW
   }

   #[derive(Debug, Serialize)]
   struct GeminiToolConfig {
       function_calling_config: GeminiFunctionCallingConfig,
   }

   #[derive(Debug, Serialize)]
   struct GeminiFunctionCallingConfig {
       mode: String, // "AUTO" | "ANY" | "NONE"
       #[serde(skip_serializing_if = "Option::is_none")]
       allowed_function_names: Option<Vec<String>>,
   }
   ```

**Example Usage**:
```rust
// Force model to use tools
let params = ModelParameters {
    tool_config: Some(ToolConfig {
        mode: ToolUseMode::Any,
        required_tool: None,
    }),
    ..Default::default()
};

// Force specific tool
let params = ModelParameters {
    tool_config: Some(ToolConfig {
        mode: ToolUseMode::Any,
        required_tool: Some("search_web".to_string()),
    }),
    ..Default::default()
};
```

#### Testing Strategy

**Unit Tests**:
- Test tool config conversion for each provider
- Test parallel tool call parsing
- Test tool use modes

**Integration Tests**:
```rust
#[tokio::test]
async fn test_forced_tool_use() {
    let model = GeminiModel::new("gemini-2.0-flash-exp", api_key);
    let tools = vec![
        ToolDefinition {
            name: "search".to_string(),
            description: "Search the web".to_string(),
            parameters: /* schema */,
        },
    ];

    let params = ModelParameters {
        tool_config: Some(ToolConfig {
            mode: ToolUseMode::Any,
            required_tool: Some("search".to_string()),
        }),
        ..Default::default()
    };

    let response = model.generate_with_tools(&messages, &tools, Some(params)).await.unwrap();
    assert!(response.tool_calls.is_some());
    assert_eq!(response.tool_calls.unwrap()[0].name, "search");
}
```

#### Dependencies
- REQ-216 (Response Metadata) helpful but not required

#### Risk Assessment
- **Low Risk**: Builds on existing orchestration layer

---

### REQ-223: Search Grounding Integration (Provider-Specific)

**Status**: PLANNED
**Priority**: LOW
**Sprint**: 5

#### User Story
```
As a Radium user,
I want models to ground their responses with Google Search results,
So that I get up-to-date information with source citations.
```

#### Why This Matters

**Current Problem**: Models only use training data cutoff knowledge. Can't access:
- Current events
- Real-time information
- Up-to-date facts

**Business Value**:
- Access to current information
- Source citations (trustworthiness)
- Reduced hallucinations
- Competitive feature (unique to Gemini)

**Technical Rationale**:
- Gemini-specific feature (Google Search integration)
- Requires REQ-216 (Response Metadata) for citations
- Provider-specific adapter pattern

#### Acceptance Criteria

**Must Have**:
- [ ] Add `GoogleSearch` tool type to Gemini provider
- [ ] Enable grounding via config flag
- [ ] Capture citations in response metadata (REQ-216)
- [ ] Display citations in CLI output
- [ ] Add `--grounding` flag to CLI

**Should Have**:
- [ ] Grounding threshold configuration
- [ ] Disable grounding for specific prompts

#### Provider Mapping

| Provider | Grounding Support | Method |
|----------|------------------|--------|
| Claude | ❌ | Not available |
| OpenAI | ❌ | Not available |
| Gemini | ✅ | Google Search tool |

**Note**: Provider-specific feature, only works with Gemini.

#### Implementation Details

**Files to Modify**:
1. `/Users/clay/Development/RAD/crates/radium-models/src/gemini.rs`
   ```rust
   #[derive(Debug, Serialize)]
   #[serde(untagged)]
   enum GeminiTool {
       FunctionDeclarations {
           function_declarations: Vec<GeminiFunctionDeclaration>,
       },
       GoogleSearch {
           google_search: GeminiGoogleSearch,
       },
   }

   #[derive(Debug, Serialize)]
   struct GeminiGoogleSearch {
       #[serde(skip_serializing_if = "Option::is_none")]
       retrieval: Option<GeminiGroundingRetrieval>,
   }

   #[derive(Debug, Serialize)]
   struct GeminiGroundingRetrieval {
       disable_attribution: bool,
       grounding_threshold: f32, // 0.0 - 1.0
   }
   ```

2. Configuration:
   ```toml
   [engines.gemini]
   model = "gemini-2.0-flash-exp"
   enable_grounding = true
   grounding_threshold = 0.3
   ```

3. CLI Flag:
   ```bash
   rad step analyze "What happened today in tech news?" --grounding
   ```

**Example Response with Citations**:
```
The latest tech news includes... [1] and ... [2]

Sources:
[1] "Tech Giant Announces..." - https://example.com/article1
[2] "New Product Launch..." - https://example.com/article2
```

#### Testing Strategy

**Integration Tests** (Gemini only):
```rust
#[tokio::test]
async fn test_gemini_grounding() {
    let model = GeminiModel::new("gemini-2.0-flash-exp", api_key);
    let messages = vec![
        ChatMessage::text("user", "What's the latest news about AI?"),
    ];

    // Enable grounding
    let mut model = model;
    model.enable_grounding(0.3);

    let response = model.generate_chat_completion(&messages, None).await.unwrap();

    // Check for citations
    let citations = response.get_citations();
    assert!(citations.is_some());
    assert!(!citations.unwrap().is_empty());
}
```

#### Dependencies
- REQ-216 (Response Metadata) for citations

#### Risk Assessment
- **Low Risk**: Provider-specific, opt-in feature

---

### REQ-224: Safety Settings Configuration (Provider-Specific)

**Status**: PLANNED
**Priority**: LOW
**Sprint**: 5

#### User Story
```
As a Radium administrator,
I want to configure safety filters for harmful content categories,
So that I can control what content the model generates based on our policies.
```

#### Why This Matters

**Current Problem**: No control over Gemini's safety filters. Cannot:
- Adjust sensitivity (BLOCK_NONE to BLOCK_LOW_AND_ABOVE)
- Disable specific categories
- Handle safety blocks programmatically

**Business Value**:
- Content policy enforcement
- Compliance requirements
- Better error handling (detect safety blocks)

**Technical Rationale**:
- Gemini-specific feature (safety ratings)
- Required for applications with content policies
- Requires REQ-216 (Response Metadata) for safety ratings

#### Acceptance Criteria

**Must Have**:
- [ ] Add `safety_settings` to Gemini request
- [ ] Support all harm categories:
  - `HARM_CATEGORY_HATE_SPEECH`
  - `HARM_CATEGORY_SEXUALLY_EXPLICIT`
  - `HARM_CATEGORY_HARASSMENT`
  - `HARM_CATEGORY_DANGEROUS_CONTENT`
- [ ] Support all thresholds:
  - `BLOCK_NONE`
  - `BLOCK_LOW_AND_ABOVE`
  - `BLOCK_MEDIUM_AND_ABOVE`
  - `BLOCK_HIGH_AND_ABOVE`
- [ ] Configuration via TOML
- [ ] Detect safety blocks in response (finish_reason = "safety")
- [ ] Display safety ratings with `--show-metadata`

**Should Have**:
- [ ] Default safe configurations
- [ ] Warning when lowering safety thresholds

#### Provider Mapping

| Provider | Safety Settings | Categories |
|----------|----------------|------------|
| Claude | ❌ | Not available |
| OpenAI | ❌ (moderation API separate) | Not available |
| Gemini | ✅ | 4 harm categories |

#### Implementation Details

**Files to Modify**:
1. `/Users/clay/Development/RAD/crates/radium-models/src/gemini.rs`
   ```rust
   #[derive(Debug, Serialize)]
   struct GeminiRequest {
       contents: Vec<GeminiContent>,
       generation_config: Option<GeminiGenerationConfig>,
       safety_settings: Option<Vec<GeminiSafetySetting>>, // NEW
   }

   #[derive(Debug, Serialize, Clone)]
   pub struct GeminiSafetySetting {
       pub category: String,   // "HARM_CATEGORY_HATE_SPEECH"
       pub threshold: String,  // "BLOCK_MEDIUM_AND_ABOVE"
   }

   impl GeminiModel {
       pub fn with_safety_settings(mut self, settings: Vec<GeminiSafetySetting>) -> Self {
           self.safety_settings = Some(settings);
           self
       }
   }
   ```

2. Configuration:
   ```toml
   [engines.gemini]
   model = "gemini-2.0-flash-exp"

   [engines.gemini.safety]
   hate_speech = "BLOCK_MEDIUM_AND_ABOVE"
   sexually_explicit = "BLOCK_MEDIUM_AND_ABOVE"
   harassment = "BLOCK_MEDIUM_AND_ABOVE"
   dangerous_content = "BLOCK_MEDIUM_AND_ABOVE"
   ```

3. Config Parsing:
   ```rust
   fn parse_safety_config(config: &GeminiEngineConfig) -> Vec<GeminiSafetySetting> {
       vec![
           GeminiSafetySetting {
               category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
               threshold: config.safety.hate_speech.clone(),
           },
           // ... other categories
       ]
   }
   ```

**Detecting Safety Blocks**:
```rust
if response.get_finish_reason() == Some("safety".to_string()) {
    let ratings = response.get_safety_ratings().unwrap();
    eprintln!("❌ Content blocked due to safety filters:");
    for rating in ratings {
        if rating.probability != "NEGLIGIBLE" {
            eprintln!("  - {}: {}", rating.category, rating.probability);
        }
    }
}
```

#### Testing Strategy

**Integration Tests**:
```rust
#[tokio::test]
async fn test_safety_settings() {
    let model = GeminiModel::new("gemini-2.0-flash-exp", api_key)
        .with_safety_settings(vec![
            GeminiSafetySetting {
                category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                threshold: "BLOCK_LOW_AND_ABOVE".to_string(),
            },
        ]);

    // Test with potentially harmful prompt
    let response = model.generate_text("Test prompt", None).await.unwrap();

    // Check safety ratings captured
    assert!(response.get_safety_ratings().is_some());
}
```

#### Dependencies
- REQ-216 (Response Metadata)

#### Risk Assessment
- **Low Risk**: Provider-specific, configuration-driven

---

### REQ-225: Structured Output Enhancement with JSON Schema

**Status**: PLANNED
**Priority**: MEDIUM
**Sprint**: 5

#### User Story
```
As a Radium developer building data extraction workflows,
I want to enforce JSON schema validation on model outputs,
So that I get reliably structured data for programmatic use.
```

#### Why This Matters

**Current Problem**: REQ-215 adds basic `response_format: "json"`, but no schema enforcement.

**Business Value**:
- Reliable structured outputs (no parsing errors)
- Data extraction workflows
- Form filling, database inserts
- Reduced post-processing

**Technical Rationale**:
- Gemini supports JSON schema (most flexible)
- Claude/OpenAI support JSON mode (less strict)
- Foundation for data pipelines

#### Acceptance Criteria

**Must Have**:
- [ ] Extend `ResponseFormat` enum with `JsonSchema(String)`
- [ ] Support JSON schema for Gemini
- [ ] Fallback to JSON mode for Claude/OpenAI
- [ ] CLI flag: `--response-schema <path>`
- [ ] Validation of response against schema
- [ ] Helpful errors when response doesn't match schema

**Should Have**:
- [ ] Schema templates (common patterns)
- [ ] Schema generation from Rust types

#### Provider Mapping

| Provider | JSON Mode | JSON Schema | Notes |
|----------|-----------|-------------|-------|
| Claude | ✅ | ❌ | JSON mode only |
| OpenAI | ✅ | ✅ | Strict mode |
| Gemini | ✅ | ✅ | Full schema support |

#### Implementation Details

**Files to Modify**:
1. `/Users/clay/Development/RAD/crates/radium-abstraction/src/lib.rs`
   ```rust
   pub enum ResponseFormat {
       Text,
       Json,
       JsonSchema(String), // JSON schema as string
   }
   ```

2. `/Users/clay/Development/RAD/crates/radium-models/src/gemini.rs`
   ```rust
   impl GeminiModel {
       fn apply_response_format(
           &self,
           config: &mut GeminiGenerationConfig,
           format: &ResponseFormat,
       ) {
           match format {
               ResponseFormat::Text => {
                   config.response_mime_type = None;
                   config.response_schema = None;
               }
               ResponseFormat::Json => {
                   config.response_mime_type = Some("application/json".to_string());
                   config.response_schema = None;
               }
               ResponseFormat::JsonSchema(schema) => {
                   config.response_mime_type = Some("application/json".to_string());
                   config.response_schema = Some(
                       serde_json::from_str(schema).unwrap()
                   );
               }
           }
       }
   }
   ```

3. CLI Usage:
   ```bash
   # JSON mode
   rad step extract "Extract key points" --response-format json

   # JSON schema
   rad step extract "Extract person details" --response-schema person.json
   ```

**Example Schema** (`person.json`):
```json
{
  "type": "object",
  "properties": {
    "name": { "type": "string" },
    "age": { "type": "integer" },
    "email": { "type": "string", "format": "email" }
  },
  "required": ["name", "age"]
}
```

**Validation**:
```rust
fn validate_response_schema(
    response: &str,
    schema: &str,
) -> Result<(), ModelError> {
    let schema_value: serde_json::Value = serde_json::from_str(schema)?;
    let response_value: serde_json::Value = serde_json::from_str(response)?;

    // Use jsonschema crate for validation
    let compiled = jsonschema::JSONSchema::compile(&schema_value)?;

    if let Err(errors) = compiled.validate(&response_value) {
        let error_messages: Vec<String> = errors
            .map(|e| e.to_string())
            .collect();
        return Err(ModelError::SchemaValidationError(error_messages.join(", ")));
    }

    Ok(())
}
```

#### Testing Strategy

**Integration Tests**:
```rust
#[tokio::test]
async fn test_json_schema_enforcement() {
    let model = GeminiModel::new("gemini-2.0-flash-exp", api_key);

    let schema = r#"{
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "integer"}
        },
        "required": ["name", "age"]
    }"#;

    let params = ModelParameters {
        response_format: Some(ResponseFormat::JsonSchema(schema.to_string())),
        ..Default::default()
    };

    let response = model.generate_text(
        "Extract: John is 30 years old",
        Some(params)
    ).await.unwrap();

    // Validate response matches schema
    let parsed: serde_json::Value = serde_json::from_str(&response.content).unwrap();
    assert_eq!(parsed["name"], "John");
    assert_eq!(parsed["age"], 30);
}
```

#### Dependencies
- REQ-215 (Extended Parameters)

#### Risk Assessment
- **Low Risk**: Builds on existing response_format

---

## PHASE 4: Optimization (Sprints 6-7)

### REQ-226: Context Caching API for Token Optimization

**Status**: PLANNED
**Priority**: MEDIUM
**Sprint**: 6

#### User Story
```
As a Radium user with repeated context,
I want to cache large prompts to reduce token costs by 50%+,
So that my applications are more cost-effective.
```

#### Why This Matters

**Current Problem**: Every request re-sends full context. Wasteful for:
- Large system prompts
- Repeated document context
- Multi-turn conversations

**Business Value**:
- 50%+ cost reduction for repeated context
- Faster response times (cached context processed faster)
- Enable larger contexts (cache expensive parts)

**Technical Rationale**:
- Claude: Prompt caching (5-minute TTL)
- Gemini: Context caching API (hours TTL)
- OpenAI: No native caching (can implement client-side)

#### Acceptance Criteria

**Must Have**:
- [ ] Implement Gemini context caching API
- [ ] Implement Claude prompt caching
- [ ] CLI commands:
  - `rad models cache-create`
  - `rad models cache-list`
  - `rad models cache-delete`
- [ ] Use cache in requests (--cache flag)
- [ ] Automatic cache creation for large contexts
- [ ] Cache metrics (hit rate, cost savings)

**Should Have**:
- [ ] Automatic cache invalidation
- [ ] Cache warming (pre-populate)

#### Provider Mapping

| Provider | Caching Support | TTL | Method |
|----------|----------------|-----|--------|
| Claude | ✅ Prompt caching | 5 min | `cache_control` in messages |
| OpenAI | ❌ | N/A | Client-side only |
| Gemini | ✅ Context caching | Hours | Cache API |

#### Implementation Details

**New File**: `/Users/clay/Development/RAD/crates/radium-models/src/gemini/cache.rs`

```rust
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct GeminiCache {
    pub name: String,          // "cachedContents/abc123"
    pub model: String,
    pub display_name: Option<String>,
    pub usage_metadata: CacheUsageMetadata,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub expire_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CacheUsageMetadata {
    pub total_token_count: u32,
}

pub struct GeminiCacheApi {
    client: reqwest::Client,
    api_key: String,
}

impl GeminiCacheApi {
    pub async fn create_cache(
        &self,
        model: &str,
        context: &[ChatMessage],
        ttl_seconds: u64,
    ) -> Result<GeminiCache, ModelError> {
        let request = json!({
            "model": format!("models/{}", model),
            "contents": context,
            "ttl": format!("{}s", ttl_seconds),
        });

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/cachedContents?key={}",
            self.api_key
        );

        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await?;

        let cache: GeminiCache = response.json().await?;
        Ok(cache)
    }

    pub async fn list_caches(&self) -> Result<Vec<GeminiCache>, ModelError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/cachedContents?key={}",
            self.api_key
        );

        let response: ListCachesResponse = self.client.get(&url).send().await?.json().await?;
        Ok(response.cached_contents)
    }

    pub async fn delete_cache(&self, name: &str) -> Result<(), ModelError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/{}?key={}",
            name, self.api_key
        );

        self.client.delete(&url).send().await?;
        Ok(())
    }
}
```

**CLI Commands**:
```bash
# Create cache
rad models cache-create \
  --model gemini-2.0-flash-exp \
  --context @large-document.txt \
  --ttl 3600

# List caches
rad models cache-list

# Use cache
rad step analyze "Summarize the document" --cache cachedContents/abc123

# Delete cache
rad models cache-delete cachedContents/abc123
```

**Automatic Caching**:
```rust
impl GeminiModel {
    async fn generate_with_auto_cache(
        &self,
        messages: &[ChatMessage],
    ) -> Result<ModelResponse, ModelError> {
        // If context > 1000 tokens, auto-cache
        let token_count = estimate_tokens(messages);

        if token_count > 1000 {
            // Create cache
            let cache_api = GeminiCacheApi::new(self.api_key.clone());
            let cache = cache_api.create_cache(&self.model_id, messages, 3600).await?;

            // Use cached context
            self.generate_with_cache(&cache.name, &[]).await
        } else {
            self.generate_chat_completion(messages, None).await
        }
    }
}
```

#### Testing Strategy

**Integration Tests**:
```rust
#[tokio::test]
async fn test_context_caching() {
    let api_key = env::var("GEMINI_API_KEY").unwrap();
    let cache_api = GeminiCacheApi::new(api_key);

    let context = vec![
        ChatMessage::text("user", "Large context...".repeat(1000)),
    ];

    // Create cache
    let cache = cache_api.create_cache("gemini-2.0-flash-exp", &context, 600).await.unwrap();

    // Use cache
    let model = GeminiModel::new("gemini-2.0-flash-exp", api_key)
        .with_cache(&cache.name);
    let response = model.generate_text("Summarize", None).await.unwrap();

    assert!(!response.content.is_empty());

    // Cleanup
    cache_api.delete_cache(&cache.name).await.unwrap();
}
```

#### Dependencies
- None

#### Risk Assessment
- **Medium Risk**: Cache lifecycle management
- **Mitigation**: Automatic expiration, cleanup

---

### REQ-227: Thinking Mode for Complex Reasoning

**Status**: PLANNED
**Priority**: LOW
**Sprint**: 6

#### User Story
```
As a Radium user solving complex problems,
I want to see the model's reasoning process,
So that I understand how it arrived at the answer.
```

#### Why This Matters

**Business Value**:
- Better answers for complex problems
- Transparency (see reasoning steps)
- Debugging (understand model behavior)

**Technical Rationale**:
- Gemini: `thinking_mode` parameter
- Claude: Extended thinking (similar concept)

#### Acceptance Criteria

**Must Have**:
- [ ] Add `thinking_mode: bool` to `ModelParameters`
- [ ] Implement for Gemini
- [ ] CLI flag: `--thinking-mode`
- [ ] Display thinking steps in output

**Should Have**:
- [ ] Option to hide thinking steps (show final answer only)

#### Implementation Details

```rust
// In ModelParameters
pub thinking_mode: Option<bool>,

// In GeminiGenerationConfig
pub thinking_mode: Option<bool>,

// CLI
rad step solve "Complex problem" --thinking-mode
```

#### Testing Strategy

**Integration Tests**:
```rust
#[tokio::test]
async fn test_thinking_mode() {
    let model = GeminiModel::new("gemini-2.0-flash-exp", api_key);
    let params = ModelParameters {
        thinking_mode: Some(true),
        ..Default::default()
    };

    let response = model.generate_text(
        "Solve: If x^2 + 5x + 6 = 0, what is x?",
        Some(params)
    ).await.unwrap();

    // Should contain reasoning steps
    assert!(response.content.contains("factor") || response.content.contains("quadratic"));
}
```

#### Dependencies
- None

#### Risk Assessment
- **Low Risk**: Simple parameter addition

---

### REQ-228: Batch Processing for Parallel Requests

**Status**: PLANNED
**Priority**: MEDIUM
**Sprint**: 7

#### User Story
```
As a Radium user with multiple prompts,
I want to process them in parallel,
So that I can maximize throughput and save time.
```

#### Why This Matters

**Business Value**:
- Faster bulk processing
- Better resource utilization
- Automation-friendly

**Technical Rationale**:
- Async/await already supports parallelism
- Just need CLI orchestration

#### Acceptance Criteria

**Must Have**:
- [ ] CLI flag: `--batch <file>` with prompts
- [ ] Configurable concurrency (--concurrency N)
- [ ] Progress indicator
- [ ] Results output (JSON or CSV)
- [ ] Error handling (continue on failure)

**Should Have**:
- [ ] Rate limiting
- [ ] Retry logic

#### Implementation Details

```rust
pub async fn execute_batch(
    batch_file: PathBuf,
    concurrency: usize,
) -> anyhow::Result<()> {
    let prompts = read_batch_file(&batch_file)?;

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut tasks = Vec::new();

    for (i, prompt) in prompts.into_iter().enumerate() {
        let permit = semaphore.clone().acquire_owned().await?;
        let task = tokio::spawn(async move {
            let result = execute_single_prompt(prompt).await;
            drop(permit);
            (i, result)
        });
        tasks.push(task);
    }

    // Collect results
    let mut results = Vec::new();
    for task in tasks {
        let (i, result) = task.await?;
        results.push((i, result));
    }

    // Output results
    output_batch_results(&results)?;
    Ok(())
}
```

**CLI Usage**:
```bash
rad step analyze --batch prompts.txt --concurrency 5 --output results.json
```

#### Testing Strategy

**E2E Tests**:
- Test batch file with 10 prompts
- Verify concurrency limit respected
- Test error handling

#### Dependencies
- None

#### Risk Assessment
- **Medium Risk**: Rate limiting needed
- **Mitigation**: Configurable concurrency, retry logic

---

### REQ-229: Code Execution Tool Integration (Provider-Specific)

**Status**: PLANNED
**Priority**: LOW
**Sprint**: 7

#### User Story
```
As a Radium user,
I want the model to write and execute code to solve problems,
So that I can get computational answers reliably.
```

#### Why This Matters

**Business Value**:
- Computational problem solving
- Data analysis
- Math verification

**Technical Rationale**:
- Gemini-specific feature
- Runs in sandbox (safe)

#### Acceptance Criteria

**Must Have**:
- [ ] Add `code_execution` tool to Gemini
- [ ] Enable via config flag
- [ ] Display code execution results

**Should Have**:
- [ ] Option to review code before execution

#### Implementation Details

```rust
// In GeminiRequest
tools: Some(vec![
    GeminiTool::CodeExecution,
]),

// Config
[engines.gemini]
enable_code_execution = true
```

#### Testing Strategy

**Integration Tests**:
```rust
#[tokio::test]
async fn test_code_execution() {
    let model = GeminiModel::new("gemini-2.0-flash-exp", api_key)
        .with_code_execution(true);

    let response = model.generate_text(
        "Calculate the sum of prime numbers up to 100",
        None
    ).await.unwrap();

    assert!(!response.content.is_empty());
}
```

#### Dependencies
- None

#### Risk Assessment
- **Low Risk**: Gemini handles execution in sandbox

---

## Summary

**Total Requirements**: 16 (REQ-214 through REQ-229)

**Phase Breakdown**:
- **Phase 1 (Foundation)**: 4 requirements - Sprint 1
- **Phase 2 (Multimodal)**: 4 requirements - Sprints 2-4
- **Phase 3 (Advanced)**: 4 requirements - Sprint 5
- **Phase 4 (Optimization)**: 4 requirements - Sprints 6-7

**Critical Path**:
1. REQ-218 (Message Structure Redesign) - MUST complete before Phase 2
2. REQ-216 (Response Metadata) - Required for REQ-223, REQ-224

**Architectural Approach**: Provider-Agnostic (80% shared, 20% provider-specific)

**Testing Strategy**: Unit → Integration → E2E for each requirement

**Risk Mitigation**: Backward compatibility via Option types and From implementations
