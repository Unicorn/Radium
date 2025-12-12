/**
 * Redis Connection Utilities
 * 
 * Functions for testing and managing Redis connections
 */

export interface ConnectionTestResult {
  success: boolean;
  message: string;
  error?: string;
}

/**
 * Test Redis connection
 */
export async function testRedisConnection(connectionUrl: string): Promise<ConnectionTestResult> {
  try {
    // Dynamic import to avoid bundling redis in client code
    const { createClient } = await import('redis');
    
    const client = createClient({
      url: connectionUrl,
      socket: {
        connectTimeout: 5000,
      },
    });

    await client.connect();
    
    // Test command
    await client.ping();
    await client.quit();

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

