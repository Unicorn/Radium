/**
 * PostgreSQL Connection Utilities
 * 
 * Functions for testing and managing PostgreSQL connections
 */

export interface ConnectionTestResult {
  success: boolean;
  message: string;
  error?: string;
}

/**
 * Test PostgreSQL connection
 */
export async function testPostgresConnection(connectionUrl: string): Promise<ConnectionTestResult> {
  try {
    // Dynamic import to avoid bundling pg in client code
    const { Client } = await import('pg');
    
    const client = new Client({
      connectionString: connectionUrl,
      connectionTimeoutMillis: 5000,
    });

    await client.connect();
    
    // Test query
    const result = await client.query('SELECT NOW() as current_time');
    await client.end();

    return {
      success: true,
      message: 'Connection successful',
    };
  } catch (error: any) {
    return {
      success: false,
      message: 'Connection failed',
      error: error.message || 'Unknown error',
    };
  }
}

