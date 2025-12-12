/**
 * Redis Activity
 * 
 * Temporal activity for executing Redis commands
 */

export interface RedisActivityInput {
  connectionUrl: string;
  command: string;
  key: string;
  value?: string;
}

export interface RedisActivityOutput {
  success: boolean;
  result?: any;
  error?: string;
}

/**
 * Execute Redis command
 */
export async function redisCommandActivity(
  input: RedisActivityInput
): Promise<RedisActivityOutput> {
  try {
    // Dynamic import to avoid bundling redis in client code
    const { createClient } = await import('redis');
    
    const client = createClient({
      url: input.connectionUrl,
      socket: {
        connectTimeout: 30000,
      },
    });

    await client.connect();

    let result: any;

    switch (input.command.toUpperCase()) {
      case 'GET':
        result = await client.get(input.key);
        break;
      case 'SET':
        if (!input.value) {
          throw new Error('SET command requires a value');
        }
        result = await client.set(input.key, input.value);
        break;
      case 'DEL':
        result = await client.del(input.key);
        break;
      case 'EXISTS':
        result = await client.exists(input.key);
        break;
      case 'INCR':
        result = await client.incr(input.key);
        break;
      case 'DECR':
        result = await client.decr(input.key);
        break;
      default:
        throw new Error(`Unsupported Redis command: ${input.command}`);
    }

    await client.quit();

    return {
      success: true,
      result,
    };
  } catch (error: any) {
    return {
      success: false,
      error: error.message || 'Unknown error',
    };
  }
}

