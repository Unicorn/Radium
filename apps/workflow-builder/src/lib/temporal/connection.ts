/**
 * Temporal Connection Manager
 * 
 * Manages connection to Temporal server and provides client instance.
 * Singleton pattern ensures only one connection is established.
 */

import { Connection, Client } from '@temporalio/client';

let clientInstance: Client | null = null;
let connectionInstance: Connection | null = null;

/**
 * Get Temporal client (creates connection if needed)
 */
export async function getTemporalClient(): Promise<Client> {
  if (clientInstance) {
    return clientInstance;
  }
  
  try {
    const connection = await getTemporalConnection();
    
    clientInstance = new Client({
      connection,
      namespace: process.env.TEMPORAL_NAMESPACE || 'default',
    });
    
    console.log('✅ Temporal client initialized');
    return clientInstance;
  } catch (error) {
    console.error('❌ Failed to create Temporal client:', error);
    throw new Error(`Failed to connect to Temporal: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Get Temporal connection (creates if needed)
 */
export async function getTemporalConnection(): Promise<Connection> {
  if (connectionInstance) {
    return connectionInstance;
  }
  
  try {
    connectionInstance = await Connection.connect({
      address: process.env.TEMPORAL_ADDRESS || 'localhost:7233',
    });
    
    console.log(`✅ Connected to Temporal at ${process.env.TEMPORAL_ADDRESS || 'localhost:7233'}`);
    return connectionInstance;
  } catch (error) {
    console.error('❌ Failed to connect to Temporal:', error);
    throw error;
  }
}

/**
 * Close Temporal connection
 * Should be called when shutting down the application
 */
export async function closeTemporalConnection() {
  if (clientInstance) {
    await clientInstance.connection.close();
    clientInstance = null;
    connectionInstance = null;
    console.log('✅ Temporal connection closed');
  }
}

/**
 * Check if Temporal connection is healthy
 */
export async function checkTemporalHealth(): Promise<boolean> {
  try {
    const client = await getTemporalClient();
    // Try to list task queues as a health check
    await client.workflow.list();
    return true;
  } catch (error) {
    console.error('❌ Temporal health check failed:', error);
    return false;
  }
}

