/**
 * tRPC API handler for Next.js App Router
 */

import { fetchRequestHandler } from '@trpc/server/adapters/fetch';
import { type NextRequest } from 'next/server';

import { appRouter } from '@/server/api/root';
import { createTRPCContext } from '@/server/api/trpc';

const handler = (req: NextRequest) => {
  console.log('🚀 [tRPC Handler] Incoming request:', {
    method: req.method,
    url: req.url,
    hasCookies: req.headers.get('cookie') ? 'YES' : 'NO',
  });
  
  return fetchRequestHandler({
    endpoint: '/api/trpc',
    req,
    router: appRouter,
    createContext: () => createTRPCContext({ headers: req.headers }),
    onError:
      process.env.NODE_ENV === 'development'
        ? ({ path, error }) => {
            console.error(
              `❌ tRPC failed on ${path ?? '<no-path>'}: ${error.message}`
            );
          }
        : undefined,
  });
};

export { handler as GET, handler as POST };

