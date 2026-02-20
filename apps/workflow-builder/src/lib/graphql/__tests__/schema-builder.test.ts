/**
 * GraphQL Schema Builder Tests
 */

import { describe, it, expect } from 'vitest';
import {
  buildGraphQLSchema,
  validateGraphQLSDL,
  extractOperationNames,
} from '../schema-builder';

describe('GraphQL Schema Builder', () => {
  describe('buildGraphQLSchema', () => {
    it('should build a valid GraphQL schema from SDL', () => {
      const sdl = `
        type Query {
          getUser(id: ID!): User
        }
        
        type User {
          id: ID!
          name: String!
          email: String!
        }
      `;

      const schema = buildGraphQLSchema(sdl);

      expect(schema).toBeDefined();
      expect(schema.getQueryType()).toBeDefined();
      expect(schema.getQueryType()?.getFields().getUser).toBeDefined();
    });

    it('should throw error for invalid SDL', () => {
      const invalidSDL = `
        type Query {
          getUser(id: ID!): User
        }
        
        # Missing User type definition
      `;

      expect(() => buildGraphQLSchema(invalidSDL)).toThrow();
    });

    it('should handle schema with queries and mutations', () => {
      const sdl = `
        type Query {
          getUser(id: ID!): User
        }
        
        type Mutation {
          createUser(name: String!, email: String!): User!
        }
        
        type User {
          id: ID!
          name: String!
          email: String!
        }
      `;

      const schema = buildGraphQLSchema(sdl);

      expect(schema.getQueryType()).toBeDefined();
      expect(schema.getMutationType()).toBeDefined();
      expect(schema.getQueryType()?.getFields().getUser).toBeDefined();
      expect(schema.getMutationType()?.getFields().createUser).toBeDefined();
    });
  });

  describe('validateGraphQLSDL', () => {
    it('should return valid for correct SDL', () => {
      const sdl = `
        type Query {
          getUser(id: ID!): User
        }
        
        type User {
          id: ID!
          name: String!
        }
      `;

      const result = validateGraphQLSDL(sdl);

      expect(result.valid).toBe(true);
      expect(result.errors).toBeUndefined();
    });

    it('should return invalid with errors for incorrect SDL', () => {
      const invalidSDL = `
        type Query {
          getUser(id: ID!): User
        }
      `;

      const result = validateGraphQLSDL(invalidSDL);

      expect(result.valid).toBe(false);
      expect(result.errors).toBeDefined();
      expect(result.errors?.length).toBeGreaterThan(0);
    });
  });

  describe('extractOperationNames', () => {
    it('should extract query names from SDL', () => {
      const sdl = `
        type Query {
          getUser(id: ID!): User
          listUsers: [User!]!
        }
        
        type User {
          id: ID!
          name: String!
        }
      `;

      const result = extractOperationNames(sdl);

      expect(result.queries).toContain('getUser');
      expect(result.queries).toContain('listUsers');
      expect(result.mutations).toEqual([]);
    });

    it('should extract mutation names from SDL', () => {
      const sdl = `
        type Mutation {
          createUser(name: String!): User!
          updateUser(id: ID!, name: String!): User!
        }
        
        type User {
          id: ID!
          name: String!
        }
      `;

      const result = extractOperationNames(sdl);

      expect(result.mutations).toContain('createUser');
      expect(result.mutations).toContain('updateUser');
      expect(result.queries).toEqual([]);
    });

    it('should extract both queries and mutations', () => {
      const sdl = `
        type Query {
          getUser(id: ID!): User
        }
        
        type Mutation {
          createUser(name: String!): User!
        }
        
        type User {
          id: ID!
          name: String!
        }
      `;

      const result = extractOperationNames(sdl);

      expect(result.queries).toContain('getUser');
      expect(result.mutations).toContain('createUser');
    });

    it('should return empty arrays when no operations exist', () => {
      const sdl = `
        type User {
          id: ID!
          name: String!
        }
      `;

      const result = extractOperationNames(sdl);

      expect(result.queries).toEqual([]);
      expect(result.mutations).toEqual([]);
    });
  });
});

