/**
 * tRPC Provider for React Query
 */

'use client';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { httpBatchLink } from '@trpc/client';
import { useState } from 'react';
import superjson from 'superjson';

import { api } from './client';

function getBaseUrl() {
  if (typeof window !== 'undefined') return '';
  if (process.env.VERCEL_URL) return `https://${process.env.VERCEL_URL}`;
  return `http://localhost:${process.env.PORT ?? 3010}`;
}

export function TRPCProvider({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(
    () => {
      console.log('🔧 [TRPCProvider] Creating QueryClient');
      return new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 5 * 1000,
            retry: 1,
            refetchOnWindowFocus: false,
            networkMode: 'always', // Force queries to run even if offline
          },
          mutations: {
            retry: 0,
            networkMode: 'always', // Force mutations to run
            onMutate: () => {
              console.log('🔄 [QueryClient] Mutation starting...');
            },
            onSuccess: (data) => {
              console.log('✅ [QueryClient] Mutation success:', data);
            },
            onError: (error) => {
              console.error('❌ [QueryClient] Mutation error:', error);
            },
          },
        },
      });
    }
  );

  const [trpcClient] = useState(() => {
    const baseUrl = getBaseUrl();
    const apiUrl = `${baseUrl}/api/trpc`;
    console.log('🔧 [TRPCProvider] Creating tRPC client with URL:', apiUrl);
    
    return api.createClient({
      transformer: superjson,
      links: [
        httpBatchLink({
          url: apiUrl,
          maxURLLength: 2083,
          // Send batch immediately (no delay)
          batchInterval: 10, // ms - very short interval
          headers() {
            console.log('🔧 [httpBatchLink] Getting headers for request');
            return {};
          },
          // CRITICAL: Include credentials (cookies) with every request
          // This ensures the auth session cookies are sent to the API
          fetch(url, options) {
            console.log('📡 [tRPC Fetch] Making request:', {
              url,
              method: options?.method,
              credentials: 'include',
              headers: options?.headers,
            });
            
            return fetch(url, {
              ...options,
              credentials: 'include',
            }).then((response) => {
              console.log('📥 [tRPC Fetch] Response received:', {
                url,
                status: response.status,
                ok: response.ok,
              });
              return response;
            }).catch((error) => {
              console.error('❌ [tRPC Fetch] Request failed:', {
                url,
                error: error.message,
              });
              throw error;
            });
          },
        }),
      ],
    });
  });

  return (
    <api.Provider client={trpcClient} queryClient={queryClient}>
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    </api.Provider>
  );
}


