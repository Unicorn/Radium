-- API Keys table
-- Stores hashed API keys for authenticating external requests to public interfaces.
-- Referenced by: apps/workflow-builder/src/server/api/routers/apiKeys.ts
--
-- RLS policies for this table are defined in 03-rls-policies.sql (which defines
-- the current_user_id() helper function needed by the policies).

CREATE TABLE IF NOT EXISTS public.api_keys (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
  project_id UUID REFERENCES public.projects(id) ON DELETE SET NULL,
  public_interface_id UUID REFERENCES public.public_interfaces(id) ON DELETE SET NULL,
  name TEXT NOT NULL,
  key_hash TEXT NOT NULL,
  key_prefix TEXT,
  is_active BOOLEAN NOT NULL DEFAULT true,
  expires_at TIMESTAMPTZ,
  last_used_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  CONSTRAINT unique_key_hash UNIQUE (key_hash),
  CONSTRAINT unique_name_per_user UNIQUE (user_id, name)
);

-- Index for key lookup (hot path for auth)
CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON public.api_keys (key_hash) WHERE is_active = true;

-- Index for user key listing
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON public.api_keys (user_id);

-- Index for filtering by public interface
CREATE INDEX IF NOT EXISTS idx_api_keys_public_interface_id ON public.api_keys (public_interface_id);

-- Enable RLS (policies are in 03-rls-policies.sql)
ALTER TABLE public.api_keys ENABLE ROW LEVEL SECURITY;

-- Grant permissions (matches pattern from 00-initial-schema.sql)
GRANT SELECT, INSERT, UPDATE, DELETE ON public.api_keys TO authenticated;
GRANT ALL ON public.api_keys TO service_role;
-- NOTE: anon role intentionally has NO access to api_keys.
-- The Rust API server uses service_role for key validation.
