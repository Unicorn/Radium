#!/bin/bash
# Seed Kong with initial routes after migration from declarative mode.
# Run once after switching to DB mode: ./scripts/seed-kong.sh
#
# This recreates all services/routes/plugins/consumers that were
# previously defined in apps/workflow-builder/supabase/volumes/api/kong.yml.
set -euo pipefail

KONG_ADMIN=${KONG_ADMIN_URL:-http://localhost:8001}

# Supabase JWT keys (must match GoTrue/PostgREST config)
ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0"
SERVICE_ROLE_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImV4cCI6MTk4MzgxMjk5Nn0.EGIM96RAZx35lJzdJsyH-qQwv8Hdp7fsn3W0YpN81IU"

echo "Waiting for Kong Admin API at $KONG_ADMIN ..."
for i in $(seq 1 30); do
  if curl -s "$KONG_ADMIN/status" > /dev/null 2>&1; then
    echo "Kong is ready."
    break
  fi
  if [ "$i" -eq 30 ]; then
    echo "ERROR: Kong did not become ready in time." >&2
    exit 1
  fi
  sleep 2
done

# ---------- Consumers ----------
echo ""
echo "=== Creating consumers ==="

curl -s -X POST "$KONG_ADMIN/consumers" -d username=DASHBOARD | jq .

curl -s -X POST "$KONG_ADMIN/consumers" -d username=anon | jq .
curl -s -X POST "$KONG_ADMIN/consumers/anon/key-auth" \
  -d "key=$ANON_KEY" | jq .

curl -s -X POST "$KONG_ADMIN/consumers" -d username=service_role | jq .
curl -s -X POST "$KONG_ADMIN/consumers/service_role/key-auth" \
  -d "key=$SERVICE_ROLE_KEY" | jq .

# ---------- ACLs ----------
echo ""
echo "=== Creating ACL groups ==="

curl -s -X POST "$KONG_ADMIN/consumers/anon/acls" -d group=anon | jq .
curl -s -X POST "$KONG_ADMIN/consumers/service_role/acls" -d group=admin | jq .

# ---------- Auth Service (open routes) ----------
echo ""
echo "=== Auth service (open routes) ==="

# auth-v1-open (/auth/v1/verify -> http://auth:9999/verify)
curl -s -X POST "$KONG_ADMIN/services" \
  -d name=auth-v1-open \
  -d url=http://auth:9999/verify | jq .
curl -s -X POST "$KONG_ADMIN/services/auth-v1-open/routes" \
  -d name=auth-v1-open \
  -d 'paths[]=/auth/v1/verify' \
  -d strip_path=true | jq .
curl -s -X POST "$KONG_ADMIN/services/auth-v1-open/plugins" -d name=cors | jq .

# auth-v1-open-callback (/auth/v1/callback -> http://auth:9999/callback)
curl -s -X POST "$KONG_ADMIN/services" \
  -d name=auth-v1-open-callback \
  -d url=http://auth:9999/callback | jq .
curl -s -X POST "$KONG_ADMIN/services/auth-v1-open-callback/routes" \
  -d name=auth-v1-open-callback \
  -d 'paths[]=/auth/v1/callback' \
  -d strip_path=true | jq .
curl -s -X POST "$KONG_ADMIN/services/auth-v1-open-callback/plugins" -d name=cors | jq .

# auth-v1-open-authorize (/auth/v1/authorize -> http://auth:9999/authorize)
curl -s -X POST "$KONG_ADMIN/services" \
  -d name=auth-v1-open-authorize \
  -d url=http://auth:9999/authorize | jq .
curl -s -X POST "$KONG_ADMIN/services/auth-v1-open-authorize/routes" \
  -d name=auth-v1-open-authorize \
  -d 'paths[]=/auth/v1/authorize' \
  -d strip_path=true | jq .
curl -s -X POST "$KONG_ADMIN/services/auth-v1-open-authorize/plugins" -d name=cors | jq .

# ---------- Auth Service (protected) ----------
echo ""
echo "=== Auth service (protected) ==="

# auth-v1 (/auth/v1/ -> http://auth:9999/) with key-auth + ACL
curl -s -X POST "$KONG_ADMIN/services" \
  -d name=auth-v1 \
  -d url=http://auth:9999/ | jq .
curl -s -X POST "$KONG_ADMIN/services/auth-v1/routes" \
  -d name=auth-v1-all \
  -d 'paths[]=/auth/v1/' \
  -d strip_path=true | jq .
curl -s -X POST "$KONG_ADMIN/services/auth-v1/plugins" -d name=cors | jq .
curl -s -X POST "$KONG_ADMIN/services/auth-v1/plugins" \
  -d name=key-auth \
  -d config.hide_credentials=false | jq .
curl -s -X POST "$KONG_ADMIN/services/auth-v1/plugins" \
  -d name=acl \
  -d config.hide_groups_header=true \
  -d 'config.allow[]=admin' \
  -d 'config.allow[]=anon' | jq .

# ---------- REST Service ----------
echo ""
echo "=== REST service ==="

curl -s -X POST "$KONG_ADMIN/services" \
  -d name=rest-v1 \
  -d url=http://rest:3000/ | jq .
curl -s -X POST "$KONG_ADMIN/services/rest-v1/routes" \
  -d name=rest-v1-all \
  -d 'paths[]=/rest/v1/' \
  -d strip_path=true | jq .
curl -s -X POST "$KONG_ADMIN/services/rest-v1/plugins" -d name=cors | jq .
curl -s -X POST "$KONG_ADMIN/services/rest-v1/plugins" \
  -d name=key-auth \
  -d config.hide_credentials=false | jq .
curl -s -X POST "$KONG_ADMIN/services/rest-v1/plugins" \
  -d name=acl \
  -d config.hide_groups_header=true \
  -d 'config.allow[]=admin' \
  -d 'config.allow[]=anon' | jq .

# ---------- Realtime Service ----------
echo ""
echo "=== Realtime service ==="

curl -s -X POST "$KONG_ADMIN/services" \
  -d name=realtime-v1 \
  -d url=http://realtime:4000/socket | jq .
curl -s -X POST "$KONG_ADMIN/services/realtime-v1/routes" \
  -d name=realtime-v1-all \
  -d 'paths[]=/realtime/v1/' \
  -d strip_path=true | jq .
curl -s -X POST "$KONG_ADMIN/services/realtime-v1/plugins" -d name=cors | jq .
curl -s -X POST "$KONG_ADMIN/services/realtime-v1/plugins" \
  -d name=key-auth \
  -d config.hide_credentials=false | jq .
curl -s -X POST "$KONG_ADMIN/services/realtime-v1/plugins" \
  -d name=acl \
  -d config.hide_groups_header=true \
  -d 'config.allow[]=admin' \
  -d 'config.allow[]=anon' | jq .

# ---------- Meta Service ----------
echo ""
echo "=== Meta service ==="

curl -s -X POST "$KONG_ADMIN/services" \
  -d name=meta \
  -d url=http://meta:8080/ | jq .
curl -s -X POST "$KONG_ADMIN/services/meta/routes" \
  -d name=meta-all \
  -d 'paths[]=/pg/' \
  -d strip_path=true | jq .
curl -s -X POST "$KONG_ADMIN/services/meta/plugins" \
  -d name=key-auth \
  -d config.hide_credentials=false | jq .
curl -s -X POST "$KONG_ADMIN/services/meta/plugins" \
  -d name=acl \
  -d config.hide_groups_header=true \
  -d 'config.allow[]=admin' | jq .

# ---------- Rust Compiler Service ----------
echo ""
echo "=== Rust compiler service ==="

curl -s -X POST "$KONG_ADMIN/services" \
  -d name=rust-compiler \
  -d url=http://rust-compiler:3000/ | jq .
curl -s -X POST "$KONG_ADMIN/services/rust-compiler/routes" \
  -d name=rust-compiler-all \
  -d 'paths[]=/api/compiler/rust/' \
  -d strip_path=true | jq .
curl -s -X POST "$KONG_ADMIN/services/rust-compiler/plugins" -d name=cors | jq .
curl -s -X POST "$KONG_ADMIN/services/rust-compiler/plugins" \
  -d name=correlation-id \
  -d config.header_name=X-Request-ID \
  -d config.generator=uuid \
  -d config.echo_downstream=true | jq .
curl -s -X POST "$KONG_ADMIN/services/rust-compiler/plugins" \
  -d name=rate-limiting \
  -d config.minute=60 \
  -d config.policy=local \
  -d config.limit_by=ip \
  -d config.hide_client_headers=false | jq .

# ---------- Radium Workflow Service ----------
echo ""
echo "=== Radium workflow service ==="

curl -s -X POST "$KONG_ADMIN/services" \
  -d name=radium-workflow \
  -d url=http://radium-workflow:3020/ | jq .
curl -s -X POST "$KONG_ADMIN/services/radium-workflow/routes" \
  -d name=radium-workflow-all \
  -d 'paths[]=/v1/workflows' \
  -d 'paths[]=/v1/components' \
  -d 'paths[]=/v1/services' \
  -d 'paths[]=/v1/projects' \
  -d strip_path=false | jq .
curl -s -X POST "$KONG_ADMIN/services/radium-workflow/plugins" -d name=cors | jq .
curl -s -X POST "$KONG_ADMIN/services/radium-workflow/plugins" \
  -d name=correlation-id \
  -d config.header_name=X-Request-ID \
  -d config.generator=uuid \
  -d config.echo_downstream=true | jq .
curl -s -X POST "$KONG_ADMIN/services/radium-workflow/plugins" \
  -d name=rate-limiting \
  -d config.minute=120 \
  -d config.policy=local \
  -d config.limit_by=ip \
  -d config.hide_client_headers=false | jq .

# Add gateway route for radium-workflow (handles /v1/gateway/* paths)
curl -s -X POST "$KONG_ADMIN/services/radium-workflow/routes" \
  -d 'paths[]=/v1/gateway' \
  -d strip_path=false | jq .

# ---------- Radium Discovery Service ----------
echo ""
echo "=== Radium discovery service ==="

curl -s -X POST "$KONG_ADMIN/services" \
  -d name=radium-discovery \
  -d url=http://radium-discovery:3030/ | jq .
curl -s -X POST "$KONG_ADMIN/services/radium-discovery/routes" \
  -d name=radium-discovery-all \
  -d 'paths[]=/v1/discover' \
  -d strip_path=false | jq .
curl -s -X POST "$KONG_ADMIN/services/radium-discovery/plugins" -d name=cors | jq .
curl -s -X POST "$KONG_ADMIN/services/radium-discovery/plugins" \
  -d name=correlation-id \
  -d config.header_name=X-Request-ID \
  -d config.generator=uuid \
  -d config.echo_downstream=true | jq .
curl -s -X POST "$KONG_ADMIN/services/radium-discovery/plugins" \
  -d name=rate-limiting \
  -d config.minute=120 \
  -d config.policy=local \
  -d config.limit_by=ip \
  -d config.hide_client_headers=false | jq .

echo ""
echo "Kong seed complete."
