# Kong API Gateway Integration

Kong is the API gateway that routes HTTP requests and applies plugins.

## Overview

The workflow builder generates Kong plugin configurations for:
- Request/response logging
- Response caching
- CORS policies
- Rate limiting

These configurations are applied during deployment, not at workflow runtime.

## Kong Concepts

### Service
A backend service that Kong routes to.

```json
{
  "name": "workflow-api",
  "url": "http://localhost:3020"
}
```

### Route
A path that maps to a service.

```json
{
  "name": "compile-route",
  "paths": ["/api/compile"],
  "service": { "id": "service-uuid" }
}
```

### Plugin
Functionality applied to routes/services.

```json
{
  "name": "http-log",
  "config": {
    "http_endpoint": "http://logging-service/logs"
  }
}
```

## Workflow Builder Components

### Kong Logging (`kong-logging`)
Configures the http-log plugin.

**Generated config:**
```typescript
const result_logging = {
  type: 'kong-logging-config',
  connector: 'production-logger'
};
```

**Kong plugin:**
```json
{
  "name": "http-log",
  "config": {
    "http_endpoint": "https://logs.example.com/ingest",
    "method": "POST",
    "content_type": "application/json"
  }
}
```

### Kong Cache (`kong-cache`)
Configures the proxy-cache plugin with Redis.

**Generated config:**
```typescript
const result_cache = {
  type: 'kong-cache-config',
  cacheKey: 'user-data',
  ttl: 3600
};
```

**Kong plugin:**
```json
{
  "name": "proxy-cache",
  "config": {
    "strategy": "redis",
    "redis": {
      "host": "redis.example.com"
    },
    "cache_ttl": 3600,
    "cache_key": ["user-data"]
  }
}
```

### Kong CORS (`kong-cors`)
Configures the cors plugin.

**Generated config:**
```typescript
const result_cors = {
  type: 'kong-cors-config',
  origins: ['https://app.example.com']
};
```

**Kong plugin:**
```json
{
  "name": "cors",
  "config": {
    "origins": ["https://app.example.com"],
    "methods": ["GET", "POST", "PUT", "DELETE"],
    "credentials": true,
    "max_age": 3600
  }
}
```

## Deployment Integration

The deployment process:
1. Reads workflow component configurations
2. Creates/updates Kong services and routes
3. Creates/updates Kong plugins
4. Applies configurations atomically

### Kong Admin API
```bash
# Create service
POST /services
{ "name": "...", "url": "..." }

# Create route
POST /services/{service}/routes
{ "paths": [...], "methods": [...] }

# Add plugin
POST /services/{service}/plugins
{ "name": "...", "config": {...} }
```

## Local Development

Run Kong locally with Docker:

```bash
docker-compose up kong kong-database
```

Default ports:
- Proxy: 8000
- Admin API: 8001
- Admin GUI: 8002

## Connector Configuration

Kong connectors store connection settings:
- Admin API URL
- Authentication credentials
- SSL certificates

Connectors are configured per-project and stored in Supabase.

## References

- [Kong Documentation](https://docs.konghq.com/)
- [Kong Plugins](https://docs.konghq.com/hub/)
- [Kong Admin API](https://docs.konghq.com/gateway/latest/admin-api/)
