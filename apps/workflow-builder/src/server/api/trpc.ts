/**
 * tRPC setup with Supabase context
 */

import { initTRPC, TRPCError } from '@trpc/server';
import superjson from 'superjson';
import { ZodError } from 'zod';
import { createClientFromHeaders } from '@/lib/supabase/server';
import type { Database } from '@/types/database';

/**
 * Create context for tRPC requests
 */
export async function createTRPCContext(opts: { headers: Headers }) {
  // Log all cookies being received
  const cookieHeader = opts.headers.get('cookie');
  console.log('🍪 [tRPC Context] Cookie header:', cookieHeader ? 'present' : 'MISSING');
  
  const supabase = createClientFromHeaders(opts.headers);
  
  // Get authenticated user
  const {
    data: { user: authUser },
    error: authError,
  } = await supabase.auth.getUser();
  
  if (authError) {
    console.error('❌ [tRPC Context] Auth error:', authError);
  }
  
  console.log('🔐 [tRPC Context] Auth user:', authUser ? {
    id: authUser.id,
    email: authUser.email,
  } : 'null');
  
  // Lazy load user record - don't block context creation
  // This will be loaded on-demand in protected procedures
  const getUserRecord = async () => {
    if (!authUser) {
      console.log('⚠️  [getUserRecord] No auth user found');
      return null;
    }
    
    console.log('📋 [getUserRecord] Looking up user record for auth_user_id:', authUser.id);
    
    const { data, error } = await supabase
      .from('users')
      .select('*')
      .eq('auth_user_id', authUser.id)
      .single();
    
    if (error) {
      console.error('❌ [getUserRecord] Error fetching user record:', error);
    } else if (data) {
      console.log('✅ [getUserRecord] Found user record:', {
        id: data.id,
        email: data.email,
        display_name: data.display_name,
      });
    } else {
      console.warn('⚠️  [getUserRecord] No user record found for auth_user_id:', authUser.id);
    }
    
    return data;
  };
  
  return {
    supabase,
    user: null as Database['public']['Tables']['users']['Row'] | null,
    authUser,
    getUserRecord,
  };
}

export type Context = Awaited<ReturnType<typeof createTRPCContext>>;

/**
 * Initialize tRPC
 */
const t = initTRPC.context<Context>().create({
  transformer: superjson,
  errorFormatter({ shape, error }) {
    return {
      ...shape,
      data: {
        ...shape.data,
        zodError:
          error.cause instanceof ZodError ? error.cause.flatten() : null,
      },
    };
  },
});

/**
 * Router and procedure helpers
 */
export const createTRPCRouter = t.router;

/**
 * Public procedure - no authentication required
 */
export const publicProcedure = t.procedure;

/**
 * Protected procedure - requires authentication
 */
export const protectedProcedure = t.procedure.use(async ({ ctx, next }) => {
  console.log('🔒 [protectedProcedure] Checking authentication...');
  
  if (!ctx.authUser) {
    console.error('❌ [protectedProcedure] No auth user in context');
    throw new TRPCError({ code: 'UNAUTHORIZED', message: 'Not authenticated' });
  }
  
  console.log('✅ [protectedProcedure] Auth user present:', ctx.authUser.email);
  
  // Load user record on-demand
  console.log('📋 [protectedProcedure] Loading user record...');
  const user = await ctx.getUserRecord();
  
  if (!user) {
    console.error('❌ [protectedProcedure] User record not found for auth_user_id:', ctx.authUser.id);
    throw new TRPCError({ 
      code: 'UNAUTHORIZED', 
      message: 'User record not found' 
    });
  }
  
  console.log('✅ [protectedProcedure] User record loaded:', {
    id: user.id,
    email: user.email,
    display_name: user.display_name,
  });
  
  return next({
    ctx: {
      ...ctx,
      user,
      authUser: ctx.authUser,
    },
  });
});

/**
 * Middleware to check user role
 */
export const hasRole = (roles: string[]) => {
  return t.middleware(async ({ ctx, next }) => {
    if (!ctx.user) {
      throw new TRPCError({ code: 'UNAUTHORIZED', message: 'Not authenticated' });
    }
    
    // Get user role
    const { data: userRole } = await ctx.supabase
      .from('user_roles')
      .select('name')
      .eq('id', ctx.user.role_id)
      .single();
    
    if (!userRole || !roles.includes(userRole.name)) {
      throw new TRPCError({ 
        code: 'FORBIDDEN', 
        message: `Requires one of: ${roles.join(', ')}` 
      });
    }
    
    return next({
      ctx: {
        ...ctx,
        role: userRole.name,
      },
    });
  });
};

/**
 * Admin-only procedure
 */
export const adminProcedure = protectedProcedure.use(hasRole(['admin']));

