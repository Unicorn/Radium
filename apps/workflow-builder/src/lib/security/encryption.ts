/**
 * Encryption Utilities
 * 
 * Encrypts and decrypts sensitive data (connector credentials, API keys, etc.)
 * using AES-256-GCM encryption.
 */

import { createCipheriv, createDecipheriv, randomBytes, scrypt } from 'crypto';
import { promisify } from 'util';

const scryptAsync = promisify(scrypt);

// Encryption algorithm
const ALGORITHM = 'aes-256-gcm';
const IV_LENGTH = 16; // 128 bits
const SALT_LENGTH = 32; // 256 bits
const TAG_LENGTH = 16; // 128 bits
const KEY_LENGTH = 32; // 256 bits

/**
 * Get encryption key from environment variable
 * Falls back to a default key for development (NOT SECURE FOR PRODUCTION)
 * 
 * In production, use:
 * - Supabase Vault
 * - AWS KMS
 * - HashiCorp Vault
 * - Or another secure key management service
 */
function getEncryptionKey(): string {
  const key = process.env.ENCRYPTION_KEY;
  
  if (!key) {
    if (process.env.NODE_ENV === 'production') {
      throw new Error('ENCRYPTION_KEY environment variable is required in production');
    }
    
    // Development fallback (NOT SECURE - only for local dev)
    console.warn('⚠️  Using default encryption key. Set ENCRYPTION_KEY in production!');
    return 'dev-encryption-key-change-in-production-min-32-chars';
  }
  
  if (key.length < 32) {
    throw new Error('ENCRYPTION_KEY must be at least 32 characters long');
  }
  
  return key;
}

/**
 * Derive encryption key from password using scrypt
 * 
 * @param password - The password/key to derive from
 * @param salt - Salt for key derivation
 * @returns Derived key buffer
 */
async function deriveKey(password: string, salt: Buffer): Promise<Buffer> {
  return (await scryptAsync(password, salt, KEY_LENGTH)) as Buffer;
}

/**
 * Encrypt data using AES-256-GCM
 * 
 * Format: salt (32 bytes) + iv (16 bytes) + encrypted data + auth tag (16 bytes)
 * 
 * @param plaintext - Data to encrypt (string or Buffer)
 * @param keyId - Optional key identifier for key rotation
 * @returns Encrypted data as Buffer
 */
export async function encrypt(
  plaintext: string | Buffer,
  keyId?: string
): Promise<Buffer> {
  const plaintextBuffer = typeof plaintext === 'string' 
    ? Buffer.from(plaintext, 'utf8') 
    : plaintext;
  
  // Generate random salt and IV
  const salt = randomBytes(SALT_LENGTH);
  const iv = randomBytes(IV_LENGTH);
  
  // Derive key from master key and salt
  const masterKey = getEncryptionKey();
  const key = await deriveKey(masterKey, salt);
  
  // Create cipher
  const cipher = createCipheriv(ALGORITHM, key, iv);
  
  // Encrypt
  const encrypted = Buffer.concat([
    cipher.update(plaintextBuffer),
    cipher.final(),
  ]);
  
  // Get authentication tag
  const tag = cipher.getAuthTag();
  
  // Combine: salt + iv + encrypted + tag
  const result = Buffer.concat([salt, iv, encrypted, tag]);
  
  return result;
}

/**
 * Decrypt data encrypted with encrypt()
 * 
 * @param encrypted - Encrypted data buffer
 * @param keyId - Optional key identifier (for key rotation support)
 * @returns Decrypted data as Buffer
 */
export async function decrypt(
  encrypted: Buffer,
  keyId?: string
): Promise<Buffer> {
  // Extract components
  const salt = encrypted.subarray(0, SALT_LENGTH);
  const iv = encrypted.subarray(SALT_LENGTH, SALT_LENGTH + IV_LENGTH);
  const tag = encrypted.subarray(encrypted.length - TAG_LENGTH);
  const encryptedData = encrypted.subarray(
    SALT_LENGTH + IV_LENGTH,
    encrypted.length - TAG_LENGTH
  );
  
  // Derive key from master key and salt
  const masterKey = getEncryptionKey();
  const key = await deriveKey(masterKey, salt);
  
  // Create decipher
  const decipher = createDecipheriv(ALGORITHM, key, iv);
  decipher.setAuthTag(tag);
  
  // Decrypt
  const decrypted = Buffer.concat([
    decipher.update(encryptedData),
    decipher.final(),
  ]);
  
  return decrypted;
}

/**
 * Encrypt a string and return as base64-encoded string
 * 
 * @param plaintext - String to encrypt
 * @param keyId - Optional key identifier
 * @returns Base64-encoded encrypted string
 */
export async function encryptString(
  plaintext: string,
  keyId?: string
): Promise<string> {
  const encrypted = await encrypt(plaintext, keyId);
  return encrypted.toString('base64');
}

/**
 * Decrypt a base64-encoded encrypted string
 * 
 * @param encryptedBase64 - Base64-encoded encrypted string
 * @param keyId - Optional key identifier
 * @returns Decrypted string
 */
export async function decryptString(
  encryptedBase64: string,
  keyId?: string
): Promise<string> {
  const encrypted = Buffer.from(encryptedBase64, 'base64');
  const decrypted = await decrypt(encrypted, keyId);
  return decrypted.toString('utf8');
}

/**
 * Encrypt JSON object
 * 
 * @param data - Object to encrypt
 * @param keyId - Optional key identifier
 * @returns Base64-encoded encrypted string
 */
export async function encryptJSON(
  data: Record<string, any>,
  keyId?: string
): Promise<string> {
  const jsonString = JSON.stringify(data);
  return encryptString(jsonString, keyId);
}

/**
 * Decrypt JSON object
 * 
 * @param encryptedBase64 - Base64-encoded encrypted string
 * @param keyId - Optional key identifier
 * @returns Decrypted object
 */
export async function decryptJSON<T = Record<string, any>>(
  encryptedBase64: string,
  keyId?: string
): Promise<T> {
  const jsonString = await decryptString(encryptedBase64, keyId);
  return JSON.parse(jsonString) as T;
}

