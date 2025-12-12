/**
 * PostgreSQL Activity
 * 
 * Temporal activity for executing PostgreSQL queries
 */

import { Client } from 'pg';

export interface PostgreSQLActivityInput {
  connectionUrl: string;
  query: string;
  parameters?: any[];
}

export interface PostgreSQLActivityOutput {
  success: boolean;
  rows?: any[];
  rowCount?: number;
  error?: string;
}

/**
 * Execute PostgreSQL query
 */
export async function postgresqlQueryActivity(
  input: PostgreSQLActivityInput
): Promise<PostgreSQLActivityOutput> {
  const client = new Client({
    connectionString: input.connectionUrl,
    connectionTimeoutMillis: 30000,
  });

  try {
    await client.connect();

    // Execute query with parameters
    const result = await client.query(
      input.query,
      input.parameters || []
    );

    await client.end();

    return {
      success: true,
      rows: result.rows,
      rowCount: result.rowCount,
    };
  } catch (error: any) {
    try {
      await client.end();
    } catch (closeError) {
      // Ignore close errors
    }

    return {
      success: false,
      error: error.message || 'Unknown error',
    };
  }
}

