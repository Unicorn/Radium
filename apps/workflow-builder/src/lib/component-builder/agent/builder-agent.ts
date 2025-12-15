/**
 * Component Builder Agent
 *
 * AI-powered agent that guides users through designing and generating
 * new workflow components through conversational interaction.
 */

import Anthropic from '@anthropic-ai/sdk';
import { KnowledgeRetrieval } from '../knowledge-base/retrieval';
import type { ProcessedRecord, SimilarComponent, SchemaDecision } from '../knowledge-base/types';
import type {
  BuilderState,
  BuilderPhase,
  ComponentRequirement,
  ComponentDesign,
  GeneratedArtifacts,
  Message,
  AgentResponse,
  BuilderAgentOptions,
  FieldDesign,
  SchemaDesign,
  ValidationRule,
  ArtifactValidation,
} from './types';

/** System prompts for different phases */
const SYSTEM_PROMPTS = {
  gathering: `You are a component designer helping create a new workflow component.
Your job is to:
1. Understand what the user wants to build
2. Ask clarifying questions about inputs, outputs, and validation
3. Reference similar components when relevant
4. Transition to design phase when requirements are clear

Guidelines:
- Be conversational but focused
- Ask 1-2 questions at a time
- Use examples from similar components
- Confirm understanding before moving forward

When you have enough information (component purpose, inputs, outputs, validation needs),
say "I now have enough information to design the schema. Let me create the design for you."`,

  designing: `You are designing a workflow component schema based on gathered requirements.

Design a complete component with:
1. Input schema (Rust struct with serde and validation)
2. Output schema (Rust struct with serde)
3. Validation rules
4. Connection rules

Use patterns from similar components.
Present the design in a clear format and ask if changes are needed.

Important:
- Use snake_case for Rust field names
- Use camelCase for TypeScript
- Include appropriate serde annotations
- Add validation derives where needed`,

  refining: `You are refining a component design based on user feedback.
Listen to the user's requested changes and update the design accordingly.
Present the updated design clearly.

When the user approves (says "looks good", "approve", "generate", etc.),
say "Great! I'll generate the code artifacts now."`,

  generating: `You are generating code artifacts for a workflow component.
Generate production-ready code following the established patterns.

Requirements:
- Rust code must compile with serde and validator derives
- TypeScript must have no 'any' types
- Include documentation comments
- Follow existing patterns from the codebase`,

  reviewing: `You are helping the user review generated artifacts.
Answer questions about the generated code.
If the user requests changes, note them for regeneration.
When finalized, confirm the component is ready to save.`,
};

/**
 * Component Builder Agent
 */
export class ComponentBuilderAgent {
  private anthropic: Anthropic;
  private knowledge: KnowledgeRetrieval;
  private state: BuilderState;
  private options: Required<BuilderAgentOptions>;

  constructor(
    knowledge: KnowledgeRetrieval,
    options: BuilderAgentOptions = {}
  ) {
    this.anthropic = new Anthropic({
      apiKey: options.apiKey,
    });
    this.knowledge = knowledge;
    this.options = {
      apiKey: options.apiKey || '',
      model: options.model || 'claude-sonnet-4-20250514',
      maxTokens: options.maxTokens || 2048,
      temperature: options.temperature || 0.7,
      debug: options.debug || false,
    };
    this.state = this.createInitialState();
  }

  /**
   * Create initial state for a new conversation
   */
  private createInitialState(): BuilderState {
    return {
      conversationId: crypto.randomUUID(),
      phase: 'gathering',
      requirement: {
        description: '',
        category: '',
        temporalType: '',
        inputs: [],
        outputs: [],
        validationRules: [],
        similarComponents: [],
        additionalContext: '',
        constraints: [],
      },
      designDraft: null,
      generatedArtifacts: null,
      messages: [],
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  /**
   * Process a user message and return a response
   */
  async chat(userMessage: string): Promise<AgentResponse> {
    const previousPhase = this.state.phase;

    // Add user message to history
    this.addMessage('user', userMessage);

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
      case 'complete':
        response = 'Component creation is complete! Start a new conversation to create another component.';
        break;
      default:
        response = 'Unknown state. Please start a new conversation.';
    }

    // Add assistant message to history
    this.addMessage('assistant', response);
    this.state.updatedAt = new Date();

    const phaseChanged = this.state.phase !== previousPhase;

    return {
      response,
      phase: this.state.phase,
      phaseChanged,
      state: {
        phase: this.state.phase,
        requirement: this.state.requirement,
        designDraft: this.state.designDraft,
        generatedArtifacts: this.state.generatedArtifacts,
      },
      suggestedActions: this.getSuggestedActions(),
    };
  }

  /**
   * Gather requirements from user
   */
  private async gatherRequirements(userMessage: string): Promise<string> {
    // Find similar components for context
    const similar = await this.knowledge.findSimilar(userMessage, 3);
    this.state.requirement.similarComponents = similar.map((s) => s.componentId);

    // Build context from similar components
    const similarContext = this.buildSimilarComponentsContext(similar);

    const systemPrompt = `${SYSTEM_PROMPTS.gathering}

${similarContext}`;

    const response = await this.callClaude(systemPrompt);

    // Check if ready to move to design
    if (this.isReadyToDesign(response)) {
      this.state.phase = 'designing';
      // Continue immediately to design
      return await this.designComponent('');
    }

    // Update requirements from conversation
    this.updateRequirementsFromMessage(userMessage);

    return response;
  }

  /**
   * Design the component schema
   */
  private async designComponent(userMessage: string): Promise<string> {
    // Get detailed knowledge from similar components
    const similarKnowledge = this.state.requirement.similarComponents
      .map((id) => this.knowledge.getComponent(id))
      .filter((r): r is ProcessedRecord => r !== undefined);

    const knowledgeContext = this.buildKnowledgeContext(similarKnowledge);
    const requirementsSummary = this.buildRequirementsSummary();

    const systemPrompt = `${SYSTEM_PROMPTS.designing}

Gathered Requirements:
${requirementsSummary}

${knowledgeContext}`;

    const response = await this.callClaude(systemPrompt);

    // Parse design from response
    this.state.designDraft = this.parseDesignFromResponse(response);
    this.state.phase = 'refining';

    return response;
  }

  /**
   * Refine the design based on feedback
   */
  private async refineDesign(userMessage: string): Promise<string> {
    // Check if user approves
    if (this.isDesignApproved(userMessage)) {
      this.state.phase = 'generating';
      return await this.generateArtifacts();
    }

    const systemPrompt = `${SYSTEM_PROMPTS.refining}

Current Design:
${JSON.stringify(this.state.designDraft, null, 2)}`;

    const response = await this.callClaude(systemPrompt);

    // Update design from response
    this.state.designDraft = this.parseDesignFromResponse(response);

    return response;
  }

  /**
   * Generate code artifacts
   */
  private async generateArtifacts(): Promise<string> {
    if (!this.state.designDraft) {
      return 'No design to generate from. Please complete the design phase first.';
    }

    // Generate each artifact
    const rustSchema = await this.generateRustSchema();
    const typescriptCode = await this.generateTypeScript();
    const testCases = await this.generateTests();
    const migrationRecord = await this.generateMigrationRecord();

    // Validate artifacts
    const validationStatus = this.validateArtifacts(rustSchema, typescriptCode);

    this.state.generatedArtifacts = {
      rustSchema,
      typescriptCode,
      testCases,
      migrationRecord,
      validationStatus,
    };

    this.state.phase = 'reviewing';

    return this.formatArtifactsSummary();
  }

  /**
   * Review artifacts and handle final approval
   */
  private async reviewArtifacts(userMessage: string): Promise<string> {
    if (this.isFinalApproval(userMessage)) {
      this.state.phase = 'complete';
      return this.finalizeComponent();
    }

    const systemPrompt = `${SYSTEM_PROMPTS.reviewing}

Generated Artifacts:
${this.formatArtifactsForReview()}`;

    return await this.callClaude(systemPrompt);
  }

  /**
   * Call Claude with the current conversation
   */
  private async callClaude(systemPrompt: string): Promise<string> {
    const messages = this.state.messages.map((m) => ({
      role: m.role as 'user' | 'assistant',
      content: m.content,
    }));

    try {
      const response = await this.anthropic.messages.create({
        model: this.options.model,
        max_tokens: this.options.maxTokens,
        system: systemPrompt,
        messages,
      });

      const content = response.content[0];
      if (content && content.type === 'text') {
        return content.text;
      }

      return 'I encountered an issue generating a response. Please try again.';
    } catch (error) {
      console.error('Claude API error:', error);
      throw new Error('Failed to communicate with AI service');
    }
  }

  /**
   * Generate Rust schema code
   */
  private async generateRustSchema(): Promise<string> {
    if (!this.state.designDraft) return '';

    const design = this.state.designDraft;
    const prompt = `Generate a complete Rust schema for this component design:
${JSON.stringify(design, null, 2)}

Requirements:
- Use serde with #[serde(rename_all = "camelCase")]
- Add validator derive for validation
- Include proper documentation comments
- Use Option for optional fields
- Add Default derive where appropriate
- Follow existing patterns from similar components

Return ONLY the Rust code, no explanations.`;

    const response = await this.anthropic.messages.create({
      model: this.options.model,
      max_tokens: 2048,
      messages: [{ role: 'user', content: prompt }],
    });

    const content = response.content[0];
    return content && content.type === 'text' ? this.extractCodeBlock(content.text, 'rust') : '';
  }

  /**
   * Generate TypeScript code
   */
  private async generateTypeScript(): Promise<string> {
    if (!this.state.designDraft) return '';

    const design = this.state.designDraft;
    const prompt = `Generate TypeScript interfaces and activity code for this component design:
${JSON.stringify(design, null, 2)}

Requirements:
- NO 'any' types - use proper types or 'unknown'
- Include JSDoc comments
- Generate proper type guards if needed
- Follow the workflow-builder patterns
- Use camelCase for field names

Return ONLY the TypeScript code, no explanations.`;

    const response = await this.anthropic.messages.create({
      model: this.options.model,
      max_tokens: 2048,
      messages: [{ role: 'user', content: prompt }],
    });

    const content = response.content[0];
    return content && content.type === 'text' ? this.extractCodeBlock(content.text, 'typescript') : '';
  }

  /**
   * Generate test cases
   */
  private async generateTests(): Promise<string> {
    if (!this.state.designDraft) return '';

    const design = this.state.designDraft;
    const prompt = `Generate comprehensive test cases for this component:
${JSON.stringify(design, null, 2)}

Include:
- Unit tests for validation
- Serialization/deserialization tests
- Edge cases
- Error cases

Return ONLY the Rust test code, no explanations.`;

    const response = await this.anthropic.messages.create({
      model: this.options.model,
      max_tokens: 2048,
      messages: [{ role: 'user', content: prompt }],
    });

    const content = response.content[0];
    return content && content.type === 'text' ? this.extractCodeBlock(content.text, 'rust') : '';
  }

  /**
   * Generate migration record YAML
   */
  private async generateMigrationRecord(): Promise<string> {
    if (!this.state.designDraft) return '';

    const design = this.state.designDraft;
    const now = new Date().toISOString();

    const record = {
      component: {
        name: design.name,
        category: design.category,
        version: '1.0.0',
        description: design.description,
        temporalType: design.temporalType,
      },
      migration: {
        migratedBy: 'Component Builder Agent',
        migrationDate: now,
        durationHours: 0,
        difficulty: 'generated',
        breakingChanges: false,
        filesCreated: [
          `src/schema/components/${design.name}.rs`,
        ],
        filesModified: [
          'src/schema/components/mod.rs',
        ],
      },
      schemaDecisions: design.decisions.map((d) => ({
        field: d.field,
        decision: d.decision,
        rationale: d.rationale,
        alternativesConsidered: d.alternativesConsidered,
      })),
      inputSchema: {
        rustStruct: design.inputSchema.rustStruct,
        typescriptInterface: design.inputSchema.typescriptInterface,
        fields: design.inputSchema.fields.map((f) => ({
          name: f.name,
          rustType: f.rustType,
          typescriptType: f.typescriptType,
          required: f.required,
          default: f.default,
          description: f.description,
        })),
        validation: [],
      },
      outputSchema: {
        rustStruct: design.outputSchema.rustStruct,
        typescriptInterface: design.outputSchema.typescriptInterface,
        fields: design.outputSchema.fields.map((f) => ({
          name: f.name,
          rustType: f.rustType,
          typescriptType: f.typescriptType,
          required: f.required,
          description: f.description,
        })),
        validation: [],
      },
      validationRules: design.validationRules.map((r) => r.rule),
      connections: {
        allowedSources: design.connections.allowedSources,
        allowedTargets: design.connections.allowedTargets,
        connectionValidation: '',
      },
      rustSchema: {
        filePath: `src/schema/components/${design.name}.rs`,
        structs: [design.inputSchema.rustStruct, design.outputSchema.rustStruct],
        enums: [],
        derives: ['Debug', 'Clone', 'Serialize', 'Deserialize', 'Validate'],
        validationImplementation: '',
      },
      typescriptTemplate: {
        templatePath: '',
        generatedCodeExample: '',
        keyPatterns: [],
      },
      testCases: [],
      lessonsLearned: {
        whatWorkedWell: ['Generated by Component Builder Agent'],
        challenges: [],
        recommendations: [],
      },
      relatedComponents: this.state.requirement.similarComponents.map((c) => ({
        component: c,
        relationship: 'Similar component',
      })),
      futureImprovements: [],
    };

    // Convert to YAML-like format
    return `# Generated by Component Builder Agent
# Date: ${now}

${JSON.stringify(record, null, 2)}`;
  }

  /**
   * Validate generated artifacts
   */
  private validateArtifacts(rustCode: string, tsCode: string): ArtifactValidation {
    const rustErrors: string[] = [];
    const tsErrors: string[] = [];

    // Basic Rust validation
    if (!rustCode.includes('struct')) {
      rustErrors.push('Missing struct definition');
    }
    if (!rustCode.includes('Serialize') && !rustCode.includes('serde')) {
      rustErrors.push('Missing serde derives');
    }

    // Basic TypeScript validation
    if (tsCode.includes(': any')) {
      tsErrors.push('Contains "any" type');
    }
    if (!tsCode.includes('interface') && !tsCode.includes('type')) {
      tsErrors.push('Missing interface/type definition');
    }

    return {
      rustValid: rustErrors.length === 0,
      rustErrors,
      typescriptValid: tsErrors.length === 0,
      typescriptErrors: tsErrors,
      isValid: rustErrors.length === 0 && tsErrors.length === 0,
    };
  }

  /**
   * Format artifacts summary for user
   */
  private formatArtifactsSummary(): string {
    if (!this.state.generatedArtifacts || !this.state.designDraft) {
      return 'No artifacts generated.';
    }

    const { rustSchema, typescriptCode, testCases, validationStatus } = this.state.generatedArtifacts;
    const design = this.state.designDraft;

    let summary = `I've generated all artifacts for the "${design.displayName}" component:

**Rust Schema** (${rustSchema.split('\n').length} lines):
\`\`\`rust
${rustSchema.substring(0, 500)}${rustSchema.length > 500 ? '\n// ... (truncated)' : ''}
\`\`\`

**TypeScript** (${typescriptCode.split('\n').length} lines):
\`\`\`typescript
${typescriptCode.substring(0, 500)}${typescriptCode.length > 500 ? '\n// ... (truncated)' : ''}
\`\`\`

**Tests**: ${testCases.split('fn test_').length - 1} test cases generated

**Validation Status**: ${validationStatus.isValid ? '✅ All checks passed' : '⚠️ Issues found'}`;

    if (!validationStatus.isValid) {
      if (validationStatus.rustErrors.length > 0) {
        summary += `\n- Rust issues: ${validationStatus.rustErrors.join(', ')}`;
      }
      if (validationStatus.typescriptErrors.length > 0) {
        summary += `\n- TypeScript issues: ${validationStatus.typescriptErrors.join(', ')}`;
      }
    }

    summary += '\n\nWould you like to review the full code, make changes, or finalize?';

    return summary;
  }

  /**
   * Format artifacts for detailed review
   */
  private formatArtifactsForReview(): string {
    if (!this.state.generatedArtifacts) return 'No artifacts to review.';

    return `
Rust Schema:
${this.state.generatedArtifacts.rustSchema}

TypeScript:
${this.state.generatedArtifacts.typescriptCode}

Tests:
${this.state.generatedArtifacts.testCases}
`;
  }

  /**
   * Finalize and save the component
   */
  private finalizeComponent(): string {
    if (!this.state.designDraft) {
      return 'No component to finalize.';
    }

    const name = this.state.designDraft.name;

    return `Component "${this.state.designDraft.displayName}" has been created!

**Files to create:**
- \`src/schema/components/${name}.rs\` - Rust schema
- \`templates/${name}.ts.hbs\` - TypeScript template
- \`tests/${name}_test.rs\` - Test cases
- \`component-records/${name}.yaml\` - Migration record

**Next steps:**
1. Review the generated code
2. Add the module to \`src/schema/components/mod.rs\`
3. Run \`cargo test\` to verify
4. The component will be available in the workflow builder

Thank you for using the Component Builder!`;
  }

  // Helper methods

  private addMessage(role: 'user' | 'assistant' | 'system', content: string): void {
    this.state.messages.push({
      role,
      content,
      timestamp: new Date(),
      phase: this.state.phase,
    });
  }

  private buildSimilarComponentsContext(similar: SimilarComponent[]): string {
    if (similar.length === 0) return '';

    const lines = ['Similar existing components:'];
    for (const s of similar) {
      const record = this.knowledge.getComponent(s.componentId);
      if (record) {
        lines.push(`- ${s.componentId}: ${record.metadata.description}`);
        lines.push(`  Similarity: ${(s.similarity * 100).toFixed(0)}% - ${s.reason}`);
      }
    }
    return lines.join('\n');
  }

  private buildKnowledgeContext(records: ProcessedRecord[]): string {
    if (records.length === 0) return '';

    const lines = ['Reference these similar component designs:'];
    for (const record of records) {
      lines.push(`\n--- ${record.id} ---`);
      lines.push(record.content.substring(0, 1000));
    }
    return lines.join('\n');
  }

  private buildRequirementsSummary(): string {
    const req = this.state.requirement;
    const lines = [
      `Description: ${req.description || 'Not specified'}`,
      `Category: ${req.category || 'To be determined'}`,
      `Temporal Type: ${req.temporalType || 'To be determined'}`,
    ];

    if (req.inputs.length > 0) {
      lines.push('Inputs:');
      for (const input of req.inputs) {
        lines.push(`  - ${input.name}: ${input.type} ${input.required ? '(required)' : '(optional)'}`);
      }
    }

    if (req.outputs.length > 0) {
      lines.push('Outputs:');
      for (const output of req.outputs) {
        lines.push(`  - ${output.name}: ${output.type}`);
      }
    }

    if (req.constraints.length > 0) {
      lines.push(`Constraints: ${req.constraints.join(', ')}`);
    }

    return lines.join('\n');
  }

  private isReadyToDesign(response: string): boolean {
    const indicators = [
      'enough information',
      'design the schema',
      'create the design',
      'ready to design',
      "let me design",
      "i'll create",
    ];
    const lower = response.toLowerCase();
    return indicators.some((i) => lower.includes(i));
  }

  private isDesignApproved(message: string): boolean {
    const approvalPhrases = [
      'looks good',
      'approve',
      'approved',
      'generate',
      'perfect',
      "let's generate",
      'go ahead',
      'proceed',
    ];
    const lower = message.toLowerCase();
    return approvalPhrases.some((p) => lower.includes(p));
  }

  private isFinalApproval(message: string): boolean {
    const finalPhrases = [
      'finalize',
      'save',
      'done',
      'complete',
      'finish',
      'create it',
    ];
    const lower = message.toLowerCase();
    return finalPhrases.some((p) => lower.includes(p));
  }

  private updateRequirementsFromMessage(message: string): void {
    // Simple heuristic updates based on message content
    if (!this.state.requirement.description) {
      this.state.requirement.description = message;
    } else {
      this.state.requirement.additionalContext += '\n' + message;
    }
  }

  private parseDesignFromResponse(response: string): ComponentDesign | null {
    // Try to extract structured design from response
    // This is a simplified parser - in production would be more robust
    const name = this.extractComponentName(response) || 'new_component';

    return {
      name,
      displayName: this.toDisplayName(name),
      category: this.state.requirement.category as 'activities' || 'activities',
      temporalType: this.state.requirement.temporalType as 'activity' || 'activity',
      description: this.state.requirement.description,
      inputSchema: this.extractSchema(response, 'input') || {
        rustStruct: `${this.toPascalCase(name)}Input`,
        typescriptInterface: `${this.toPascalCase(name)}Input`,
        fields: [],
      },
      outputSchema: this.extractSchema(response, 'output') || {
        rustStruct: `${this.toPascalCase(name)}Output`,
        typescriptInterface: `${this.toPascalCase(name)}Output`,
        fields: [],
      },
      validationRules: [],
      connections: {
        allowedSources: ['*'],
        allowedTargets: ['*'],
        multipleInputs: false,
        multipleOutputs: true,
      },
      decisions: [],
      appliedPatterns: [],
    };
  }

  private extractComponentName(response: string): string | null {
    const match = response.match(/component[:\s]+["']?(\w+)["']?/i);
    return match && match[1] ? match[1].toLowerCase() : null;
  }

  private extractSchema(response: string, type: 'input' | 'output'): SchemaDesign | null {
    // Simplified extraction - would need more robust parsing
    const fields: FieldDesign[] = [];

    // Look for field patterns like "- name: String"
    const fieldPattern = /[-*]\s*(\w+):\s*(\w+(?:<[^>]+>)?)/g;
    let match;
    while ((match = fieldPattern.exec(response)) !== null) {
      const fieldName = match[1];
      const rustType = match[2];
      if (fieldName && rustType) {
        fields.push({
          name: fieldName,
          rustType: rustType,
          typescriptType: this.rustToTypeScript(rustType),
          required: !rustType.startsWith('Option'),
          serde: [],
          description: '',
        });
      }
    }

    if (fields.length === 0) return null;

    const pascalName = this.toPascalCase(this.extractComponentName(response) || 'Component');
    return {
      rustStruct: `${pascalName}${type === 'input' ? 'Input' : 'Output'}`,
      typescriptInterface: `${pascalName}${type === 'input' ? 'Input' : 'Output'}`,
      fields,
    };
  }

  private extractCodeBlock(text: string, language: string): string {
    const pattern = new RegExp(`\`\`\`${language}\\s*([\\s\\S]*?)\`\`\``, 'i');
    const match = text.match(pattern);
    return match && match[1] ? match[1].trim() : text;
  }

  private rustToTypeScript(rustType: string): string {
    const mapping: Record<string, string> = {
      'String': 'string',
      'str': 'string',
      'bool': 'boolean',
      'i32': 'number',
      'i64': 'number',
      'u32': 'number',
      'u64': 'number',
      'f32': 'number',
      'f64': 'number',
    };

    if (mapping[rustType]) return mapping[rustType];
    if (rustType.startsWith('Option<')) {
      const inner = rustType.slice(7, -1);
      return `${this.rustToTypeScript(inner)} | undefined`;
    }
    if (rustType.startsWith('Vec<')) {
      const inner = rustType.slice(4, -1);
      return `${this.rustToTypeScript(inner)}[]`;
    }
    if (rustType.startsWith('HashMap<')) {
      return 'Record<string, unknown>';
    }
    return 'unknown';
  }

  private toPascalCase(str: string): string {
    return str
      .split(/[_-]/)
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
      .join('');
  }

  private toDisplayName(str: string): string {
    return str
      .split(/[_-]/)
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
      .join(' ');
  }

  private getSuggestedActions(): string[] {
    switch (this.state.phase) {
      case 'gathering':
        return [
          'Describe what your component does',
          'Specify input fields',
          'Specify output fields',
        ];
      case 'designing':
        return ['Wait for design...'];
      case 'refining':
        return [
          'Request changes',
          'Approve the design',
          'Add more fields',
        ];
      case 'generating':
        return ['Wait for generation...'];
      case 'reviewing':
        return [
          'Review the code',
          'Request changes',
          'Finalize and save',
        ];
      case 'complete':
        return ['Start new component'];
      default:
        return [];
    }
  }

  // Public getters

  getState(): BuilderState {
    return { ...this.state };
  }

  getPhase(): BuilderPhase {
    return this.state.phase;
  }

  getConversationId(): string {
    return this.state.conversationId;
  }

  reset(): void {
    this.state = this.createInitialState();
  }
}
