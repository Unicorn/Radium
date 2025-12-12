/**
 * Supabase client for server-side usage (Server Components, Server Actions, Route Handlers)
 */

import { createServerClient, type CookieOptions } from '@supabase/ssr';
import { cookies } from 'next/headers';
import type { Database } from '@/types/database';

/**
 * Create Supabase client for use in API routes with explicit headers
 */
export function createClientFromHeaders(headers: Headers) {
  // Parse cookies from header string and decode values
  const parseCookies = (cookieHeader: string | null): Map<string, string> => {
    const cookies = new Map<string, string>();
    if (!cookieHeader) return cookies;
    
    cookieHeader.split(';').forEach(cookie => {
      const [name, ...rest] = cookie.trim().split('=');
      if (name) {
        const value = rest.join('=');
        // Decode the cookie value - Supabase stores session data as URL-encoded JSON
        try {
          cookies.set(name, decodeURIComponent(value));
        } catch (e) {
          // If decode fails, use the raw value
          cookies.set(name, value);
        }
      }
    });
    return cookies;
  };

  const cookieMap = parseCookies(headers.get('cookie'));

  return createServerClient<Database>(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!,
    {
      cookies: {
        get(name: string) {
          return cookieMap.get(name);
        },
        set(name: string, value: string, options: CookieOptions) {
          // Can't set cookies in API routes from here
        },
        remove(name: string, options: CookieOptions) {
          // Can't remove cookies in API routes from here
        },
      },
    }
  );
}

/**
 * Create Supabase client for Server Components/Actions
 */
export async function createClient() {
  const cookieStore = await cookies();

  return createServerClient<Database>(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!,
    {
      cookies: {
        get(name: string) {
          return cookieStore.get(name)?.value;
        },
        set(name: string, value: string, options: CookieOptions) {
          try {
            cookieStore.set({ name, value, ...options });
          } catch (error) {
            // Server component, can't set cookies
          }
        },
        remove(name: string, options: CookieOptions) {
          try {
            cookieStore.set({ name, value: '', ...options });
          } catch (error) {
            // Server component, can't remove cookies
          }
        },
      },
    }
  );
}

