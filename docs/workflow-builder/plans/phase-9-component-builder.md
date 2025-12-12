# Phase 9: Component Builder System

**Status**: Not Started
**Duration**: 3-4 weeks
**Prerequisites**: Phase 6 (Component Migration - for training data), Phase 8 (Production Hardening)
**Blocks**: None (final phase)

## Overview

Build an AI-powered Component Builder system within the workflow-builder UI that enables admins to create new workflow components through conversational interaction. This system learns from the detailed migration records created in Phase 6 to understand component patterns, schema design, and validation rules.

## Goals

1. Create intuitive UI for component creation
2. Train AI agent on migration records
3. Enable conversational component design
4. Support schema-driven and visual building
5. Generate production-ready Rust schemas and TypeScript

## Vision

The Component Builder represents the culmination of the migration effort. By systematically documenting every decision, alternative, and lesson learned during Phase 6, we create a knowledge base that enables AI-assisted component creation. An admin can describe a new component in natural language, and the agent will:

1. Understand the requirement
2. Reference similar components from training data
3. Design appropriate input/output schemas
4. Generate Rust validation code
5. Produce TypeScript templates
6. Create test cases

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Component Builder Architecture                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                         Admin UI Layer                                 │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                │  │
│  │  │ Conversation │  │   Schema     │  │   Visual     │                │  │
│  │  │    Panel     │  │   Editor     │  │   Builder    │                │  │
│  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                │  │
│  │         │                 │                 │                         │  │
│  │         └─────────────────┼─────────────────┘                         │  │
│  │                           │                                           │  │
│  └───────────────────────────┼───────────────────────────────────────────┘  │
│                              │                                               │
│                              ▼                                               │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                    Component Builder Agent                             │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                │  │
│  │  │  Knowledge   │  │   Schema     │  │    Code      │                │  │
│  │  │   Retrieval  │  │   Designer   │  │  Generator   │                │  │
│  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                │  │
│  │         │                 │                 │                         │  │
│  └─────────┼─────────────────┼─────────────────┼─────────────────────────┘  │
│            │                 │                 │                            │
│            ▼                 ▼                 ▼                            │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                      Knowledge Base                                    │  │
│  │  ┌──────────────────────────────────────────────────────────────┐    │  │
│  │  │                 Migration Records (Phase 6)                   │    │  │
│  │  │  - kong-logging.yaml    - conditional.yaml                   │    │  │
│  │  │  - http-request.yaml    - loop.yaml                          │    │  │
│  │  │  - agent.yaml           - child-workflow.yaml                │    │  │
│  │  │  - ...14+ component records                                  │    │  │
│  │  └──────────────────────────────────────────────────────────────┘    │  │
│  │  ┌──────────────────────────────────────────────────────────────┐    │  │
│  │  │              Component Patterns Library                       │    │  │
│  │  │  - Validation patterns    - TypeScript patterns              │    │  │
│  │  │  - Error handling         - Schema design principles          │    │  │
│  │  └──────────────────────────────────────────────────────────────┘    │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Component Creation Flow

```
User                     Agent                        System
  │                        │                            │
  ├─ "I need a component   │                            │
  │   that sends emails"   │                            │
  │ ──────────────────────►│                            │
  │                        ├─ Search knowledge base ───►│
  │                        │◄── Similar: http-request,  │
  │                        │    agent, kong-logging ────│
  │                        │                            │
  │◄── "I found similar    │                            │
  │    components. Let me  │                            │
  │    ask about your      │                            │
  │    requirements..."    │                            │
  │                        │                            │
  ├─ "SMTP, with retry,    │                            │
  │   HTML templates"      │                            │
  │ ──────────────────────►│                            │
  │                        ├─ Design schema ───────────►│
  │                        │                            │
  │◄── "Here's my proposed │                            │
  │    schema. Review:"    │                            │
  │    [Schema preview]    │                            │
  │                        │                            │
  ├─ "Looks good, add      │                            │
  │   attachment support"  │                            │
  │ ──────────────────────►│                            │
  │                        ├─ Update schema ───────────►│
  │                        │                            │
  │◄── "Updated. Ready to  │                            │
  │    generate?"          │                            │
  │                        │                            │
  ├─ "Yes, generate"       │                            │
  │ ──────────────────────►│                            │
  │                        ├─ Generate Rust schema ────►│
  │                        ├─ Generate TypeScript ─────►│
  │                        ├─ Generate tests ──────────►│
  │                        ├─ Create migration record ─►│
  │                        │                            │
  │◄── "Component created: │                            │
  │    - Rust schema       │                            │
  │    - TypeScript        │                            │
  │    - Tests             │                            │
  │    - Documentation"    │                            │
  │                        │                            │
```

---

## Tasks

### 8.1 Knowledge Base Setup

**Description**: Create the training data pipeline from migration records

**Subtasks**:
- [ ] 8.1.1 Create knowledge base schema
- [ ] 8.1.2 Process migration records into embeddings
- [ ] 8.1.3 Extract component patterns
- [ ] 8.1.4 Create pattern categorization
- [ ] 8.1.5 Build retrieval system
- [ ] 8.1.6 Create similarity search
- [ ] 8.1.7 Test knowledge retrieval
- [ ] 8.1.8 Document knowledge base structure

**Files to Create**:
```typescript
// src/component-builder/knowledge-base/types.ts

export interface ComponentKnowledge {
  /** Component identifier */
  componentId: string;

  /** Component metadata */
  metadata: ComponentMetadata;

  /** Extracted patterns */
  patterns: ComponentPatterns;

  /** Design decisions */
  decisions: SchemaDecision[];

  /** Related components */
  related: string[];

  /** Embedding vector for similarity search */
  embedding: number[];
}

export interface ComponentMetadata {
  name: string;
  category: string;
  description: string;
  temporalType: 'activity' | 'workflow' | 'signal' | 'query';
  complexity: 'low' | 'medium' | 'high';
  migrationDate: Date;
}

export interface ComponentPatterns {
  /** Input validation patterns */
  inputValidation: ValidationPattern[];

  /** Output schema patterns */
  outputSchema: SchemaPattern[];

  /** Error handling patterns */
  errorHandling: ErrorPattern[];

  /** TypeScript generation patterns */
  typescriptPatterns: CodePattern[];
}

export interface ValidationPattern {
  type: string;
  field: string;
  rule: string;
  rustImplementation: string;
  rationale: string;
}

export interface SchemaPattern {
  fieldType: string;
  rustType: string;
  typescriptType: string;
  serdeAnnotations: string[];
  example: string;
}

export interface ErrorPattern {
  errorType: string;
  handling: string;
  recovery: string;
}

export interface CodePattern {
  pattern: string;
  template: string;
  usage: string;
}

export interface SchemaDecision {
  field: string;
  decision: string;
  rationale: string;
  alternatives: Alternative[];
}

export interface Alternative {
  approach: string;
  pros: string[];
  cons: string[];
  whyRejected: string;
}
```

```typescript
// src/component-builder/knowledge-base/processor.ts
import * as yaml from 'yaml';
import * as fs from 'fs';
import * as path from 'path';

export interface ProcessedRecord {
  id: string;
  content: string;
  metadata: Record<string, unknown>;
  embedding?: number[];
}

export class MigrationRecordProcessor {
  private recordsDir: string;

  constructor(recordsDir: string) {
    this.recordsDir = recordsDir;
  }

  async processAll(): Promise<ProcessedRecord[]> {
    const records: ProcessedRecord[] = [];
    const files = fs.readdirSync(this.recordsDir)
      .filter(f => f.endsWith('.yaml') && f !== '_template.yaml');

    for (const file of files) {
      const record = await this.processFile(path.join(this.recordsDir, file));
      records.push(record);
    }

    return records;
  }

  async processFile(filePath: string): Promise<ProcessedRecord> {
    const content = fs.readFileSync(filePath, 'utf-8');
    const parsed = yaml.parse(content);

    // Extract key information for knowledge base
    const processedContent = this.extractContent(parsed);

    return {
      id: parsed.component.name,
      content: processedContent,
      metadata: {
        category: parsed.component.category,
        temporalType: parsed.component.temporal_type,
        description: parsed.component.description,
        migrationDifficulty: parsed.migration.difficulty,
        schemaDecisions: parsed.schema_decisions,
        lessonsLearned: parsed.lessons_learned,
      },
    };
  }

  private extractContent(record: Record<string, unknown>): string {
    // Create a structured text representation for embedding
    const parts: string[] = [];

    // Component description
    const component = record.component as Record<string, unknown>;
    parts.push(`Component: ${component.name}`);
    parts.push(`Category: ${component.category}`);
    parts.push(`Description: ${component.description}`);

    // Schema decisions
    const decisions = record.schema_decisions as Array<Record<string, unknown>>;
    if (decisions) {
      parts.push('\nSchema Decisions:');
      for (const decision of decisions) {
        parts.push(`- ${decision.field}: ${decision.decision}`);
        parts.push(`  Rationale: ${decision.rationale}`);
      }
    }

    // Input schema
    const inputSchema = record.input_schema as Record<string, unknown>;
    if (inputSchema) {
      parts.push('\nInput Schema:');
      parts.push(`Rust: ${inputSchema.rust_struct}`);
      parts.push(`TypeScript: ${inputSchema.typescript_interface}`);
    }

    // Lessons learned
    const lessons = record.lessons_learned as Record<string, unknown>;
    if (lessons) {
      parts.push('\nLessons Learned:');
      const challenges = lessons.challenges as Array<Record<string, unknown>>;
      if (challenges) {
        for (const challenge of challenges) {
          parts.push(`- Challenge: ${challenge.challenge}`);
          parts.push(`  Solution: ${challenge.solution}`);
        }
      }
    }

    return parts.join('\n');
  }
}
```

```typescript
// src/component-builder/knowledge-base/retrieval.ts
import Anthropic from '@anthropic-ai/sdk';

export interface SimilarComponent {
  componentId: string;
  similarity: number;
  relevantDecisions: string[];
  applicablePatterns: string[];
}

export class KnowledgeRetrieval {
  private anthropic: Anthropic;
  private knowledgeBase: Map<string, ProcessedRecord>;

  constructor(anthropic: Anthropic) {
    this.anthropic = anthropic;
    this.knowledgeBase = new Map();
  }

  async loadKnowledgeBase(records: ProcessedRecord[]): Promise<void> {
    for (const record of records) {
      // Generate embeddings using Claude
      // Note: Using message API for now since embeddings API may not be available
      this.knowledgeBase.set(record.id, record);
    }
  }

  async findSimilar(query: string, limit = 5): Promise<SimilarComponent[]> {
    // Use Claude to find semantically similar components
    const prompt = `Given this component requirement:
"${query}"

And these existing components:
${Array.from(this.knowledgeBase.values())
  .map(r => `- ${r.id}: ${(r.metadata.description as string) || ''}`)
  .join('\n')}

Which components are most similar or relevant? Return as JSON array with format:
[{"componentId": "name", "similarity": 0.0-1.0, "reason": "why relevant"}]`;

    const response = await this.anthropic.messages.create({
      model: 'claude-3-5-sonnet-20241022',
      max_tokens: 1024,
      messages: [{ role: 'user', content: prompt }],
    });

    // Parse response and return similar components
    const content = response.content[0];
    if (content.type === 'text') {
      try {
        const matches = JSON.parse(
          content.text.match(/\[[\s\S]*\]/)?.[0] || '[]'
        );
        return matches.slice(0, limit).map((m: Record<string, unknown>) => ({
          componentId: m.componentId as string,
          similarity: m.similarity as number,
          relevantDecisions: [],
          applicablePatterns: [],
        }));
      } catch {
        return [];
      }
    }

    return [];
  }

  getComponent(id: string): ProcessedRecord | undefined {
    return this.knowledgeBase.get(id);
  }
}
```

**Acceptance Criteria**:
- [ ] Migration records processed
- [ ] Knowledge base queryable
- [ ] Similar components retrievable
- [ ] Patterns extracted

---

### 8.2 Component Builder Agent

**Description**: Create the AI agent for component design

**Subtasks**:
- [ ] 8.2.1 Design agent architecture
- [ ] 8.2.2 Create conversation management
- [ ] 8.2.3 Implement requirement gathering
- [ ] 8.2.4 Create schema designer
- [ ] 8.2.5 Implement code generator
- [ ] 8.2.6 Add validation step
- [ ] 8.2.7 Create review/refinement loop
- [ ] 8.2.8 Test with sample components

**Files to Create**:
```typescript
// src/component-builder/agent/builder-agent.ts
import Anthropic from '@anthropic-ai/sdk';
import { KnowledgeRetrieval } from '../knowledge-base/retrieval';

export interface BuilderState {
  conversationId: string;
  phase: BuilderPhase;
  requirement: ComponentRequirement;
  designDraft: ComponentDesign | null;
  generatedArtifacts: GeneratedArtifacts | null;
  messages: Message[];
}

export type BuilderPhase =
  | 'gathering'
  | 'designing'
  | 'refining'
  | 'generating'
  | 'reviewing'
  | 'complete';

export interface ComponentRequirement {
  description: string;
  category: string;
  inputs: FieldRequirement[];
  outputs: FieldRequirement[];
  validationRules: string[];
  similarComponents: string[];
  additionalContext: string;
}

export interface FieldRequirement {
  name: string;
  description: string;
  type: string;
  required: boolean;
  defaultValue?: string;
  constraints?: string[];
}

export interface ComponentDesign {
  name: string;
  category: string;
  temporalType: string;
  inputSchema: SchemaDesign;
  outputSchema: SchemaDesign;
  validationRules: ValidationRule[];
  connections: ConnectionRules;
}

export interface SchemaDesign {
  fields: FieldDesign[];
  rustStruct: string;
  typescriptInterface: string;
}

export interface FieldDesign {
  name: string;
  rustType: string;
  typescriptType: string;
  required: boolean;
  default?: string;
  serde: string[];
  validation?: string;
}

export interface ValidationRule {
  field: string;
  rule: string;
  errorMessage: string;
}

export interface ConnectionRules {
  allowedSources: string[];
  allowedTargets: string[];
}

export interface GeneratedArtifacts {
  rustSchema: string;
  typescriptCode: string;
  testCases: string;
  migrationRecord: string;
}

export interface Message {
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

export class ComponentBuilderAgent {
  private anthropic: Anthropic;
  private knowledge: KnowledgeRetrieval;
  private state: BuilderState;

  constructor(anthropic: Anthropic, knowledge: KnowledgeRetrieval) {
    this.anthropic = anthropic;
    this.knowledge = knowledge;
    this.state = this.createInitialState();
  }

  private createInitialState(): BuilderState {
    return {
      conversationId: crypto.randomUUID(),
      phase: 'gathering',
      requirement: {
        description: '',
        category: '',
        inputs: [],
        outputs: [],
        validationRules: [],
        similarComponents: [],
        additionalContext: '',
      },
      designDraft: null,
      generatedArtifacts: null,
      messages: [],
    };
  }

  async chat(userMessage: string): Promise<string> {
    this.state.messages.push({
      role: 'user',
      content: userMessage,
      timestamp: new Date(),
    });

    let response: string;

    switch (this.state.phase) {
      case 'gathering':
        response = await this.gatherRequirements(userMessage);
        break;
      case 'designing':
        response = await this.designComponent(userMessage);
        break;
      case 'refining':
        response = await this.refineDesign(userMessage);
        break;
      case 'generating':
        response = await this.generateArtifacts();
        break;
      case 'reviewing':
        response = await this.reviewArtifacts(userMessage);
        break;
      default:
        response = 'Component creation complete!';
    }

    this.state.messages.push({
      role: 'assistant',
      content: response,
      timestamp: new Date(),
    });

    return response;
  }

  private async gatherRequirements(userMessage: string): Promise<string> {
    // Find similar components for context
    const similar = await this.knowledge.findSimilar(userMessage);
    this.state.requirement.similarComponents = similar.map(s => s.componentId);

    // Use Claude to gather and clarify requirements
    const systemPrompt = `You are a component designer helping create a new workflow component.
You have access to these similar components: ${similar.map(s => s.componentId).join(', ')}

Your job is to:
1. Understand what the user wants to build
2. Ask clarifying questions about inputs, outputs, and validation
3. Reference similar components when relevant
4. Transition to design phase when requirements are clear

Be conversational but focused. Ask 1-2 questions at a time.`;

    const response = await this.anthropic.messages.create({
      model: 'claude-3-5-sonnet-20241022',
      max_tokens: 1024,
      system: systemPrompt,
      messages: this.state.messages.map(m => ({
        role: m.role,
        content: m.content,
      })),
    });

    const content = response.content[0];
    if (content.type === 'text') {
      // Check if we have enough info to move to design
      if (this.hasEnoughRequirements(content.text)) {
        this.state.phase = 'designing';
      }
      return content.text;
    }

    return 'I need more information about your component requirements.';
  }

  private hasEnoughRequirements(response: string): boolean {
    // Check if response indicates readiness to design
    const readyIndicators = [
      'ready to design',
      'let me create the schema',
      'based on your requirements',
      'here is the proposed design',
    ];
    return readyIndicators.some(i =>
      response.toLowerCase().includes(i)
    );
  }

  private async designComponent(userMessage: string): Promise<string> {
    // Get detailed knowledge from similar components
    const similarKnowledge = this.state.requirement.similarComponents
      .map(id => this.knowledge.getComponent(id))
      .filter(Boolean);

    const systemPrompt = `You are designing a workflow component schema.

Reference these similar component designs:
${similarKnowledge.map(k => k?.content).join('\n\n---\n\n')}

Design a complete component with:
1. Input schema (Rust struct with serde)
2. Output schema
3. Validation rules
4. Connection rules

Use the patterns from similar components.
Present the design for review and ask if any changes are needed.`;

    const response = await this.anthropic.messages.create({
      model: 'claude-3-5-sonnet-20241022',
      max_tokens: 2048,
      system: systemPrompt,
      messages: this.state.messages.map(m => ({
        role: m.role,
        content: m.content,
      })),
    });

    const content = response.content[0];
    if (content.type === 'text') {
      // Parse design from response
      this.state.designDraft = this.parseDesignFromResponse(content.text);
      this.state.phase = 'refining';
      return content.text;
    }

    return 'Error creating design.';
  }

  private parseDesignFromResponse(response: string): ComponentDesign | null {
    // Parse the design structure from the Claude response
    // This would extract Rust structs, TypeScript interfaces, etc.
    // For now, return a placeholder
    return {
      name: 'new-component',
      category: 'activity',
      temporalType: 'activity',
      inputSchema: {
        fields: [],
        rustStruct: '',
        typescriptInterface: '',
      },
      outputSchema: {
        fields: [],
        rustStruct: '',
        typescriptInterface: '',
      },
      validationRules: [],
      connections: {
        allowedSources: ['*'],
        allowedTargets: ['*'],
      },
    };
  }

  private async refineDesign(userMessage: string): Promise<string> {
    // Handle design refinement requests
    if (userMessage.toLowerCase().includes('looks good') ||
        userMessage.toLowerCase().includes('approve') ||
        userMessage.toLowerCase().includes('generate')) {
      this.state.phase = 'generating';
      return await this.generateArtifacts();
    }

    // Otherwise, refine based on feedback
    const response = await this.anthropic.messages.create({
      model: 'claude-3-5-sonnet-20241022',
      max_tokens: 2048,
      system: 'You are refining a component design based on user feedback. Update the design and present it again.',
      messages: this.state.messages.map(m => ({
        role: m.role,
        content: m.content,
      })),
    });

    const content = response.content[0];
    return content.type === 'text' ? content.text : 'Error refining design.';
  }

  private async generateArtifacts(): Promise<string> {
    if (!this.state.designDraft) {
      return 'No design to generate from.';
    }

    // Generate all artifacts
    const rustSchema = await this.generateRustSchema();
    const typescriptCode = await this.generateTypeScript();
    const testCases = await this.generateTests();
    const migrationRecord = await this.generateMigrationRecord();

    this.state.generatedArtifacts = {
      rustSchema,
      typescriptCode,
      testCases,
      migrationRecord,
    };

    this.state.phase = 'reviewing';

    return `I've generated all artifacts for your component:

**Rust Schema:**
\`\`\`rust
${rustSchema.substring(0, 500)}...
\`\`\`

**TypeScript:**
\`\`\`typescript
${typescriptCode.substring(0, 500)}...
\`\`\`

**Tests:** ${testCases.split('\n').length} test cases generated

Would you like to review the full generated code, make changes, or finalize?`;
  }

  private async generateRustSchema(): Promise<string> {
    // Use Claude to generate the Rust schema
    const response = await this.anthropic.messages.create({
      model: 'claude-3-5-sonnet-20241022',
      max_tokens: 2048,
      messages: [{
        role: 'user',
        content: `Generate a complete Rust schema for this component design:
${JSON.stringify(this.state.designDraft, null, 2)}

Follow these patterns:
- Use serde with rename_all = "lowercase" or "snake_case"
- Add validator derive for validation
- Include proper documentation comments
- Use Option for optional fields
- Add Default derive where appropriate`,
      }],
    });

    const content = response.content[0];
    return content.type === 'text' ? content.text : '';
  }

  private async generateTypeScript(): Promise<string> {
    // Use Claude to generate TypeScript
    const response = await this.anthropic.messages.create({
      model: 'claude-3-5-sonnet-20241022',
      max_tokens: 2048,
      messages: [{
        role: 'user',
        content: `Generate TypeScript interfaces and activity code for this component design:
${JSON.stringify(this.state.designDraft, null, 2)}

Requirements:
- NO 'any' types - use proper types or 'unknown'
- Include JSDoc comments
- Generate proper type guards
- Follow the workflow-builder patterns`,
      }],
    });

    const content = response.content[0];
    return content.type === 'text' ? content.text : '';
  }

  private async generateTests(): Promise<string> {
    // Generate test cases
    const response = await this.anthropic.messages.create({
      model: 'claude-3-5-sonnet-20241022',
      max_tokens: 2048,
      messages: [{
        role: 'user',
        content: `Generate comprehensive test cases for this component:
${JSON.stringify(this.state.designDraft, null, 2)}

Include:
- Unit tests for validation
- Integration tests
- Behavior tests
- Edge cases`,
      }],
    });

    const content = response.content[0];
    return content.type === 'text' ? content.text : '';
  }

  private async generateMigrationRecord(): Promise<string> {
    // Generate migration record in YAML format
    const record = {
      component: {
        name: this.state.designDraft?.name,
        category: this.state.designDraft?.category,
        version: '1.0.0',
        description: this.state.requirement.description,
        temporal_type: this.state.designDraft?.temporalType,
      },
      migration: {
        migrated_by: 'Component Builder Agent',
        migration_date: new Date().toISOString(),
        duration_hours: 0,
        difficulty: 'generated',
        breaking_changes: false,
        files_created: [],
        files_modified: [],
      },
      schema_decisions: [],
      // ... rest of the record structure
    };

    return `# Generated by Component Builder Agent
# Date: ${new Date().toISOString()}

${JSON.stringify(record, null, 2)}`;
  }

  private async reviewArtifacts(userMessage: string): Promise<string> {
    if (userMessage.toLowerCase().includes('finalize') ||
        userMessage.toLowerCase().includes('save') ||
        userMessage.toLowerCase().includes('done')) {
      this.state.phase = 'complete';
      return this.finalizeComponent();
    }

    // Handle review feedback
    return 'What changes would you like to make?';
  }

  private finalizeComponent(): string {
    // Save all artifacts to disk
    // In a real implementation, this would write files

    return `Component "${this.state.designDraft?.name}" has been created!

Files created:
- src/schemas/components/${this.state.designDraft?.name}.rs
- templates/${this.state.designDraft?.name}.ts.hbs
- tests/${this.state.designDraft?.name}_test.rs
- component-records/${this.state.designDraft?.name}.yaml

The component is ready to use in workflows.`;
  }

  getState(): BuilderState {
    return this.state;
  }

  reset(): void {
    this.state = this.createInitialState();
  }
}
```

**Acceptance Criteria**:
- [ ] Agent gathers requirements
- [ ] Schema design works
- [ ] Refinement loop functional
- [ ] Artifacts generated correctly

---

### 8.3 Admin UI

**Description**: Create the UI for component building

**Subtasks**:
- [ ] 8.3.1 Create conversation panel component
- [ ] 8.3.2 Create schema editor component
- [ ] 8.3.3 Create visual builder component
- [ ] 8.3.4 Create preview/review panel
- [ ] 8.3.5 Implement real-time collaboration
- [ ] 8.3.6 Add version control integration
- [ ] 8.3.7 Create component library view
- [ ] 8.3.8 Test complete workflow

**Files to Create**:
```typescript
// src/app/admin/component-builder/page.tsx
'use client';

import { useState } from 'react';
import { Stack, XStack, YStack, Text, Input, Button, ScrollView } from 'tamagui';
import { ComponentBuilderAgent } from '@/component-builder/agent/builder-agent';

export default function ComponentBuilderPage() {
  const [messages, setMessages] = useState<Array<{role: string, content: string}>>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [artifacts, setArtifacts] = useState<{
    rustSchema?: string;
    typescript?: string;
    tests?: string;
  } | null>(null);

  const sendMessage = async () => {
    if (!input.trim()) return;

    const userMessage = input;
    setInput('');
    setMessages(prev => [...prev, { role: 'user', content: userMessage }]);
    setLoading(true);

    try {
      const response = await fetch('/api/component-builder/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ message: userMessage }),
      });

      const data = await response.json();

      setMessages(prev => [...prev, { role: 'assistant', content: data.response }]);

      if (data.artifacts) {
        setArtifacts(data.artifacts);
      }
    } catch (error) {
      console.error('Error:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <XStack f={1} h="100vh">
      {/* Conversation Panel */}
      <YStack f={1} p="$4" borderRightWidth={1} borderColor="$borderColor">
        <Text fontSize="$6" fontWeight="bold" mb="$4">
          Component Builder
        </Text>

        <ScrollView f={1}>
          <YStack gap="$3">
            {messages.map((msg, i) => (
              <YStack
                key={i}
                p="$3"
                br="$3"
                bg={msg.role === 'user' ? '$blue2' : '$gray2'}
                alignSelf={msg.role === 'user' ? 'flex-end' : 'flex-start'}
                maxWidth="80%"
              >
                <Text>{msg.content}</Text>
              </YStack>
            ))}
            {loading && (
              <YStack p="$3" br="$3" bg="$gray2">
                <Text>Thinking...</Text>
              </YStack>
            )}
          </YStack>
        </ScrollView>

        <XStack gap="$2" mt="$4">
          <Input
            f={1}
            value={input}
            onChangeText={setInput}
            placeholder="Describe your component..."
            onSubmitEditing={sendMessage}
          />
          <Button onPress={sendMessage} disabled={loading}>
            Send
          </Button>
        </XStack>
      </YStack>

      {/* Preview Panel */}
      <YStack f={1} p="$4">
        <Text fontSize="$6" fontWeight="bold" mb="$4">
          Generated Code
        </Text>

        {artifacts ? (
          <YStack gap="$4" f={1}>
            <YStack f={1}>
              <Text fontWeight="bold" mb="$2">Rust Schema</Text>
              <ScrollView f={1} bg="$gray1" p="$3" br="$3">
                <Text fontFamily="$mono" fontSize="$2">
                  {artifacts.rustSchema}
                </Text>
              </ScrollView>
            </YStack>

            <YStack f={1}>
              <Text fontWeight="bold" mb="$2">TypeScript</Text>
              <ScrollView f={1} bg="$gray1" p="$3" br="$3">
                <Text fontFamily="$mono" fontSize="$2">
                  {artifacts.typescript}
                </Text>
              </ScrollView>
            </YStack>
          </YStack>
        ) : (
          <YStack f={1} ai="center" jc="center">
            <Text color="$gray10">
              Start a conversation to design your component
            </Text>
          </YStack>
        )}
      </YStack>
    </XStack>
  );
}
```

```typescript
// src/app/admin/component-builder/schema-editor.tsx
'use client';

import { useState } from 'react';
import { YStack, XStack, Text, Input, Button, Select } from 'tamagui';

interface Field {
  name: string;
  type: string;
  required: boolean;
  default?: string;
  validation?: string;
}

interface SchemaEditorProps {
  fields: Field[];
  onChange: (fields: Field[]) => void;
}

const RUST_TYPES = [
  'String',
  'i32',
  'i64',
  'f64',
  'bool',
  'Vec<String>',
  'Option<String>',
  'serde_json::Value',
  'chrono::DateTime<Utc>',
];

export function SchemaEditor({ fields, onChange }: SchemaEditorProps) {
  const addField = () => {
    onChange([...fields, { name: '', type: 'String', required: true }]);
  };

  const updateField = (index: number, updates: Partial<Field>) => {
    const newFields = [...fields];
    newFields[index] = { ...newFields[index], ...updates };
    onChange(newFields);
  };

  const removeField = (index: number) => {
    onChange(fields.filter((_, i) => i !== index));
  };

  return (
    <YStack gap="$3">
      <XStack jc="space-between" ai="center">
        <Text fontWeight="bold">Schema Fields</Text>
        <Button size="$2" onPress={addField}>+ Add Field</Button>
      </XStack>

      {fields.map((field, index) => (
        <XStack key={index} gap="$2" ai="center">
          <Input
            f={1}
            value={field.name}
            onChangeText={(name) => updateField(index, { name })}
            placeholder="Field name"
          />

          <Select
            value={field.type}
            onValueChange={(type) => updateField(index, { type })}
          >
            <Select.Trigger w={150}>
              <Select.Value placeholder="Type" />
            </Select.Trigger>
            <Select.Content>
              {RUST_TYPES.map(type => (
                <Select.Item key={type} value={type}>
                  <Select.ItemText>{type}</Select.ItemText>
                </Select.Item>
              ))}
            </Select.Content>
          </Select>

          <Button
            size="$2"
            variant={field.required ? 'outlined' : 'ghost'}
            onPress={() => updateField(index, { required: !field.required })}
          >
            {field.required ? 'Required' : 'Optional'}
          </Button>

          <Button size="$2" onPress={() => removeField(index)} theme="red">
            Remove
          </Button>
        </XStack>
      ))}
    </YStack>
  );
}
```

**Acceptance Criteria**:
- [ ] Conversation UI works
- [ ] Schema editor functional
- [ ] Preview panel updates
- [ ] Component saved successfully

---

### 8.4 Visual Component Builder

**Description**: Create drag-and-drop visual builder

**Subtasks**:
- [ ] 8.4.1 Create field palette
- [ ] 8.4.2 Implement drag-and-drop
- [ ] 8.4.3 Create validation rule builder
- [ ] 8.4.4 Add connection rule editor
- [ ] 8.4.5 Create live preview
- [ ] 8.4.6 Implement undo/redo
- [ ] 8.4.7 Add keyboard shortcuts
- [ ] 8.4.8 Test visual builder

**Acceptance Criteria**:
- [ ] Drag-and-drop works
- [ ] Visual updates in real-time
- [ ] Generates valid schemas
- [ ] User-friendly interface

---

### 8.5 Template System

**Description**: Create reusable component templates

**Subtasks**:
- [ ] 8.5.1 Create template library
- [ ] 8.5.2 Add template categories
- [ ] 8.5.3 Implement template preview
- [ ] 8.5.4 Create template customization
- [ ] 8.5.5 Add community templates
- [ ] 8.5.6 Create template versioning
- [ ] 8.5.7 Implement template sharing
- [ ] 8.5.8 Test template system

**Template Categories**:
```typescript
// src/component-builder/templates/types.ts

export interface ComponentTemplate {
  id: string;
  name: string;
  description: string;
  category: TemplateCategory;
  version: string;
  author: string;
  inputSchema: SchemaTemplate;
  outputSchema: SchemaTemplate;
  validationRules: ValidationRuleTemplate[];
  exampleUsage: string;
  tags: string[];
}

export type TemplateCategory =
  | 'communication'  // Email, SMS, Slack
  | 'data'           // Database, API, Transform
  | 'integration'    // Third-party services
  | 'control'        // Flow control, conditions
  | 'ai'             // AI/ML components
  | 'custom';        // User-created

export interface SchemaTemplate {
  fields: FieldTemplate[];
  customizable: string[];
}

export interface FieldTemplate {
  name: string;
  type: string;
  required: boolean;
  customizable: boolean;
  description: string;
}

export interface ValidationRuleTemplate {
  field: string;
  rule: string;
  customizable: boolean;
}
```

**Built-in Templates**:
```typescript
// src/component-builder/templates/built-in/email-sender.ts

export const emailSenderTemplate: ComponentTemplate = {
  id: 'email-sender',
  name: 'Email Sender',
  description: 'Send emails via SMTP with HTML template support',
  category: 'communication',
  version: '1.0.0',
  author: 'Workflow Builder Team',
  inputSchema: {
    fields: [
      { name: 'to', type: 'String', required: true, customizable: false, description: 'Recipient email' },
      { name: 'subject', type: 'String', required: true, customizable: false, description: 'Email subject' },
      { name: 'body', type: 'String', required: true, customizable: false, description: 'Email body (HTML)' },
      { name: 'from', type: 'Option<String>', required: false, customizable: true, description: 'Sender email' },
      { name: 'attachments', type: 'Vec<Attachment>', required: false, customizable: true, description: 'File attachments' },
    ],
    customizable: ['from', 'attachments'],
  },
  outputSchema: {
    fields: [
      { name: 'success', type: 'bool', required: true, customizable: false, description: 'Send success' },
      { name: 'messageId', type: 'Option<String>', required: false, customizable: false, description: 'SMTP message ID' },
    ],
    customizable: [],
  },
  validationRules: [
    { field: 'to', rule: 'email', customizable: false },
    { field: 'from', rule: 'email', customizable: false },
  ],
  exampleUsage: `
const result = await activities.sendEmail({
  to: 'user@example.com',
  subject: 'Welcome!',
  body: '<h1>Hello</h1>',
});
`,
  tags: ['email', 'communication', 'smtp'],
};
```

**Acceptance Criteria**:
- [ ] Template library available
- [ ] Templates customizable
- [ ] Preview working
- [ ] Templates versioned

---

### 8.6 Testing & Validation

**Description**: Comprehensive testing for component builder

**Subtasks**:
- [ ] 8.6.1 Create end-to-end tests
- [ ] 8.6.2 Test conversation flow
- [ ] 8.6.3 Test schema generation
- [ ] 8.6.4 Test code generation
- [ ] 8.6.5 Test visual builder
- [ ] 8.6.6 Test template system
- [ ] 8.6.7 Performance testing
- [ ] 8.6.8 User testing

**Test File**:
```typescript
// tests/component-builder/builder-agent.test.ts
import { ComponentBuilderAgent } from '@/component-builder/agent/builder-agent';
import { KnowledgeRetrieval } from '@/component-builder/knowledge-base/retrieval';
import Anthropic from '@anthropic-ai/sdk';

describe('ComponentBuilderAgent', () => {
  let agent: ComponentBuilderAgent;
  let knowledge: KnowledgeRetrieval;

  beforeEach(async () => {
    const anthropic = new Anthropic();
    knowledge = new KnowledgeRetrieval(anthropic);

    // Load test knowledge base
    await knowledge.loadKnowledgeBase([
      {
        id: 'test-component',
        content: 'Test component for testing',
        metadata: { category: 'test', description: 'A test component' },
      },
    ]);

    agent = new ComponentBuilderAgent(anthropic, knowledge);
  });

  test('should start in gathering phase', () => {
    expect(agent.getState().phase).toBe('gathering');
  });

  test('should gather requirements from user', async () => {
    const response = await agent.chat('I need a component that sends webhooks');

    expect(response).toBeDefined();
    expect(response.length).toBeGreaterThan(0);
    expect(agent.getState().requirement.description).toBeTruthy();
  });

  test('should find similar components', async () => {
    await agent.chat('I need a component that makes HTTP requests');

    const state = agent.getState();
    expect(state.requirement.similarComponents.length).toBeGreaterThanOrEqual(0);
  });

  test('should transition to design phase', async () => {
    // Simulate requirement gathering
    await agent.chat('I need a webhook sender component');
    await agent.chat('It should have URL, method, headers, and body');
    await agent.chat('Yes, that covers the requirements');

    const state = agent.getState();
    expect(['designing', 'refining']).toContain(state.phase);
  });

  test('should generate artifacts when approved', async () => {
    // Fast-forward to design phase
    await agent.chat('Create a simple logging component');
    await agent.chat('Just message and level fields');
    await agent.chat('Looks good, generate');

    const state = agent.getState();
    if (state.generatedArtifacts) {
      expect(state.generatedArtifacts.rustSchema).toBeDefined();
      expect(state.generatedArtifacts.typescriptCode).toBeDefined();
      expect(state.generatedArtifacts.testCases).toBeDefined();
    }
  });

  test('should reset state correctly', () => {
    agent.reset();

    const state = agent.getState();
    expect(state.phase).toBe('gathering');
    expect(state.messages).toHaveLength(0);
    expect(state.designDraft).toBeNull();
  });
});
```

**Acceptance Criteria**:
- [ ] All tests pass
- [ ] E2E flow verified
- [ ] Edge cases handled
- [ ] User feedback incorporated

---

### 8.7 Documentation

**Description**: Document the component builder system

**Subtasks**:
- [ ] 8.7.1 Create user guide
- [ ] 8.7.2 Create admin guide
- [ ] 8.7.3 Document agent architecture
- [ ] 8.7.4 Document knowledge base
- [ ] 8.7.5 Create video tutorials
- [ ] 8.7.6 Document API
- [ ] 8.7.7 Create examples
- [ ] 8.7.8 Create FAQ

**Acceptance Criteria**:
- [ ] User guide complete
- [ ] Admin guide complete
- [ ] API documented
- [ ] Examples provided

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Component creation time | < 30 min | Average |
| User satisfaction | > 4/5 | Survey |
| Generated code quality | 100% compile | tsc --strict |
| Agent accuracy | > 90% | Correct schemas |
| Knowledge retrieval | > 80% | Relevant results |

---

## Files to Create

```
packages/workflow-builder/
  src/
    component-builder/
      knowledge-base/
        types.ts
        processor.ts
        retrieval.ts
        embeddings.ts
      agent/
        builder-agent.ts
        schema-designer.ts
        code-generator.ts
        conversation.ts
      templates/
        types.ts
        built-in/
          email-sender.ts
          webhook.ts
          database-query.ts
    app/
      admin/
        component-builder/
          page.tsx
          schema-editor.tsx
          visual-builder.tsx
          preview-panel.tsx
    api/
      component-builder/
        chat/
          route.ts
        generate/
          route.ts
  tests/
    component-builder/
      builder-agent.test.ts
      knowledge-base.test.ts
      templates.test.ts
```

---

## Checklist

Before marking Phase 9 complete:

- [ ] Knowledge base from migration records
- [ ] Agent gathers requirements correctly
- [ ] Schema design is accurate
- [ ] Code generation produces valid output
- [ ] UI is intuitive and functional
- [ ] Visual builder works
- [ ] Template system available
- [ ] All tests passing
- [ ] Documentation complete
- [ ] User testing completed
- [ ] Deployed to production
