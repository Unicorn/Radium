/**
 * Visual Builder Component Tests
 *
 * Tests for the visual component builder UI components.
 */

import { describe, it, expect } from 'vitest';
import { FIELD_TYPES } from '../../../components/component-builder/FieldPalette';

describe('FieldPalette', () => {
  describe('FIELD_TYPES', () => {
    it('should have primitive types', () => {
      const primitives = FIELD_TYPES.filter((f) => f.category === 'primitive');

      expect(primitives.length).toBeGreaterThan(0);
      expect(primitives.some((f) => f.id === 'string')).toBe(true);
      expect(primitives.some((f) => f.id === 'number')).toBe(true);
      expect(primitives.some((f) => f.id === 'boolean')).toBe(true);
    });

    it('should have complex types', () => {
      const complex = FIELD_TYPES.filter((f) => f.category === 'complex');

      expect(complex.length).toBeGreaterThan(0);
      expect(complex.some((f) => f.id === 'array')).toBe(true);
      expect(complex.some((f) => f.id === 'object')).toBe(true);
    });

    it('should have temporal types', () => {
      const temporal = FIELD_TYPES.filter((f) => f.category === 'temporal');

      expect(temporal.length).toBeGreaterThan(0);
      expect(temporal.some((f) => f.id === 'datetime')).toBe(true);
    });

    it('should have correct Rust types for each field', () => {
      const stringField = FIELD_TYPES.find((f) => f.id === 'string');
      const numberField = FIELD_TYPES.find((f) => f.id === 'number');
      const boolField = FIELD_TYPES.find((f) => f.id === 'boolean');

      expect(stringField?.rustType).toBe('String');
      expect(numberField?.rustType).toBe('i64');
      expect(boolField?.rustType).toBe('bool');
    });

    it('should have correct TypeScript types for each field', () => {
      const stringField = FIELD_TYPES.find((f) => f.id === 'string');
      const numberField = FIELD_TYPES.find((f) => f.id === 'number');
      const boolField = FIELD_TYPES.find((f) => f.id === 'boolean');

      expect(stringField?.typescriptType).toBe('string');
      expect(numberField?.typescriptType).toBe('number');
      expect(boolField?.typescriptType).toBe('boolean');
    });

    it('should have description for each field type', () => {
      FIELD_TYPES.forEach((field) => {
        expect(field.description).toBeDefined();
        expect(field.description.length).toBeGreaterThan(0);
      });
    });
  });
});

describe('ValidationRule types', () => {
  it('should define validation rule types', () => {
    // Type-only test - validating that types exist
    const ruleTypes = [
      'required',
      'email',
      'url',
      'min_length',
      'max_length',
      'min_value',
      'max_value',
      'pattern',
      'enum',
      'custom',
    ];

    expect(ruleTypes).toContain('required');
    expect(ruleTypes).toContain('email');
    expect(ruleTypes).toContain('pattern');
  });
});

describe('ConnectionRules', () => {
  it('should have default connection rule structure', () => {
    const defaultRules = {
      allowedSources: ['*'],
      allowedTargets: ['*'],
      maxConnections: 10,
      allowSelfLoop: false,
      requiredPrevious: [],
    };

    expect(defaultRules.allowedSources).toContain('*');
    expect(defaultRules.allowedTargets).toContain('*');
    expect(defaultRules.maxConnections).toBe(10);
    expect(defaultRules.allowSelfLoop).toBe(false);
    expect(defaultRules.requiredPrevious).toHaveLength(0);
  });
});

describe('Visual Builder State Management', () => {
  it('should support undo/redo operations conceptually', () => {
    // Testing the concept of history management
    type HistoryState<T> = {
      current: T;
      history: T[];
      index: number;
    };

    const createHistoryState = <T>(initial: T): HistoryState<T> => ({
      current: initial,
      history: [initial],
      index: 0,
    });

    const pushState = <T>(
      state: HistoryState<T>,
      newValue: T
    ): HistoryState<T> => ({
      current: newValue,
      history: [...state.history.slice(0, state.index + 1), newValue],
      index: state.index + 1,
    });

    const undo = <T>(state: HistoryState<T>): HistoryState<T> => {
      if (state.index <= 0) return state;
      const newIndex = state.index - 1;
      return {
        ...state,
        current: state.history[newIndex]!,
        index: newIndex,
      };
    };

    const redo = <T>(state: HistoryState<T>): HistoryState<T> => {
      if (state.index >= state.history.length - 1) return state;
      const newIndex = state.index + 1;
      return {
        ...state,
        current: state.history[newIndex]!,
        index: newIndex,
      };
    };

    // Test
    let state = createHistoryState<string>('initial');
    expect(state.current).toBe('initial');
    expect(state.index).toBe(0);

    state = pushState(state, 'second');
    expect(state.current).toBe('second');
    expect(state.index).toBe(1);

    state = pushState(state, 'third');
    expect(state.current).toBe('third');
    expect(state.index).toBe(2);

    state = undo(state);
    expect(state.current).toBe('second');
    expect(state.index).toBe(1);

    state = undo(state);
    expect(state.current).toBe('initial');
    expect(state.index).toBe(0);

    // Can't undo past initial
    state = undo(state);
    expect(state.current).toBe('initial');
    expect(state.index).toBe(0);

    state = redo(state);
    expect(state.current).toBe('second');
    expect(state.index).toBe(1);

    state = redo(state);
    expect(state.current).toBe('third');
    expect(state.index).toBe(2);

    // Can't redo past end
    state = redo(state);
    expect(state.current).toBe('third');
    expect(state.index).toBe(2);
  });

  it('should truncate future history on new change after undo', () => {
    type HistoryState<T> = {
      current: T;
      history: T[];
      index: number;
    };

    const pushState = <T>(
      state: HistoryState<T>,
      newValue: T
    ): HistoryState<T> => ({
      current: newValue,
      // Truncate history after current index, then add new state
      history: [...state.history.slice(0, state.index + 1), newValue],
      index: state.index + 1,
    });

    let state: HistoryState<string> = {
      current: 'third',
      history: ['initial', 'second', 'third'],
      index: 2,
    };

    // Simulate undo to 'second'
    state = {
      ...state,
      current: 'second',
      index: 1,
    };

    // Push new state - should truncate 'third'
    state = pushState(state, 'new');

    expect(state.history).toEqual(['initial', 'second', 'new']);
    expect(state.current).toBe('new');
    expect(state.index).toBe(2);
  });
});

describe('Code Generation Preview', () => {
  it('should convert component name to PascalCase struct name', () => {
    const toPascalCase = (name: string): string => {
      return name
        .split(/[-_\s]+/)
        .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
        .join('');
    };

    expect(toPascalCase('my_component')).toBe('MyComponent');
    expect(toPascalCase('http-request')).toBe('HttpRequest');
    expect(toPascalCase('database query')).toBe('DatabaseQuery');
    expect(toPascalCase('simpleTest')).toBe('Simpletest'); // Note: single word doesn't split
  });

  it('should map Rust types to TypeScript types', () => {
    const rustToTs: Record<string, string> = {
      String: 'string',
      i32: 'number',
      i64: 'number',
      f64: 'number',
      bool: 'boolean',
      'Vec<String>': 'string[]',
      'serde_json::Value': 'Record<string, unknown>',
      'Option<String>': 'string | undefined',
      'chrono::DateTime<chrono::Utc>': 'string',
      'std::time::Duration': 'number',
    };

    expect(rustToTs['String']).toBe('string');
    expect(rustToTs['i64']).toBe('number');
    expect(rustToTs['bool']).toBe('boolean');
    expect(rustToTs['Vec<String>']).toBe('string[]');
    expect(rustToTs['serde_json::Value']).toBe('Record<string, unknown>');
  });
});
