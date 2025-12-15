# Radium Workflow Compiler - Operations Runbook

## Overview

This runbook provides operational procedures for the Radium Workflow Compiler service, including deployment, monitoring, troubleshooting, and incident response.

## Service Information

- **Service Name**: radium-workflow
- **Port**: 3000
- **Health Endpoints**:
  - Liveness: `GET /health/live`
  - Readiness: `GET /health/ready`
  - Metrics: `GET /metrics`

## Quick Reference

### Key Metrics

| Metric | Description | Alert Threshold |
|--------|-------------|-----------------|
| `compilation_requests_total` | Total compilation requests | - |
| `compilation_duration_seconds` | Compilation latency | p99 > 5s |
| `compilation_errors_total` | Failed compilations | > 10/min |
| `rate_limit_exceeded_total` | Rate limit hits | > 100/min |
| `cache_hit_ratio` | Cache effectiveness | < 50% |

### Key Logs

```bash
# View recent logs
kubectl logs -l app=radium-workflow --tail=100

# Stream logs
kubectl logs -l app=radium-workflow -f

# Filter for errors
kubectl logs -l app=radium-workflow | grep -i error
```

## Deployment Procedures

### Standard Deployment

```bash
# Build and tag image
docker build -t radium-workflow:$VERSION ./crates/radium-workflow

# Push to registry
docker tag radium-workflow:$VERSION ghcr.io/your-org/radium-workflow:$VERSION
docker push ghcr.io/your-org/radium-workflow:$VERSION

# Deploy to Kubernetes
cd crates/radium-workflow/deploy/kubernetes
kustomize edit set image radium-workflow=ghcr.io/your-org/radium-workflow:$VERSION
kubectl apply -k .
```

### Rolling Update

```bash
# Update deployment
kubectl set image deployment/radium-workflow \
  radium-workflow=ghcr.io/your-org/radium-workflow:$NEW_VERSION

# Monitor rollout
kubectl rollout status deployment/radium-workflow

# Verify pods are healthy
kubectl get pods -l app=radium-workflow
```

### Rollback Procedure

```bash
# View rollout history
kubectl rollout history deployment/radium-workflow

# Rollback to previous version
kubectl rollout undo deployment/radium-workflow

# Rollback to specific revision
kubectl rollout undo deployment/radium-workflow --to-revision=2
```

## Scaling Procedures

### Manual Scaling

```bash
# Scale to specific replica count
kubectl scale deployment/radium-workflow --replicas=5

# Verify scaling
kubectl get pods -l app=radium-workflow
```

### Auto-scaling Configuration

```bash
# View current HPA status
kubectl get hpa radium-workflow

# Adjust HPA limits
kubectl patch hpa radium-workflow -p '{"spec":{"maxReplicas":20}}'
```

## Health Checks

### Verify Service Health

```bash
# Check liveness
kubectl exec -it $(kubectl get pod -l app=radium-workflow -o jsonpath='{.items[0].metadata.name}') -- \
  wget -qO- http://localhost:3000/health/live

# Check readiness
kubectl exec -it $(kubectl get pod -l app=radium-workflow -o jsonpath='{.items[0].metadata.name}') -- \
  wget -qO- http://localhost:3000/health/ready

# Get metrics
kubectl exec -it $(kubectl get pod -l app=radium-workflow -o jsonpath='{.items[0].metadata.name}') -- \
  wget -qO- http://localhost:3000/metrics
```

### Pod Health Overview

```bash
# Check all pod statuses
kubectl get pods -l app=radium-workflow -o wide

# Describe unhealthy pods
kubectl describe pod -l app=radium-workflow | grep -A5 "Conditions:"

# Check recent events
kubectl get events --sort-by='.lastTimestamp' | grep radium-workflow
```

## Common Operations

### View Configuration

```bash
# Get current ConfigMap
kubectl get configmap radium-workflow-config -o yaml

# Get environment variables from running pod
kubectl exec -it $(kubectl get pod -l app=radium-workflow -o jsonpath='{.items[0].metadata.name}') -- env
```

### Update Configuration

```bash
# Edit ConfigMap
kubectl edit configmap radium-workflow-config

# Restart pods to pick up changes
kubectl rollout restart deployment/radium-workflow
```

### Cache Management

```bash
# Check cache stats (from metrics)
kubectl exec -it $(kubectl get pod -l app=radium-workflow -o jsonpath='{.items[0].metadata.name}') -- \
  wget -qO- http://localhost:3000/metrics | grep cache

# Clear cache (if endpoint available)
# POST /admin/cache/clear (requires admin auth)
```

## Incident Response

### High Latency (p99 > 5s)

1. **Check current load**
   ```bash
   kubectl top pods -l app=radium-workflow
   kubectl get hpa radium-workflow
   ```

2. **Check for resource exhaustion**
   ```bash
   kubectl describe pod -l app=radium-workflow | grep -A10 "Resources:"
   ```

3. **Scale up if needed**
   ```bash
   kubectl scale deployment/radium-workflow --replicas=10
   ```

4. **Check for problematic workflows**
   - Review recent compilation requests in logs
   - Look for workflows with many components

### High Error Rate

1. **Identify error types**
   ```bash
   kubectl logs -l app=radium-workflow --tail=500 | grep -i error | sort | uniq -c
   ```

2. **Check validation errors vs system errors**
   - Validation errors: User input issues
   - System errors: Infrastructure problems

3. **Review specific errors**
   ```bash
   kubectl logs -l app=radium-workflow | grep "compilation_failed" | tail -20
   ```

### Service Unavailable

1. **Check pod status**
   ```bash
   kubectl get pods -l app=radium-workflow
   kubectl describe pods -l app=radium-workflow
   ```

2. **Check node health**
   ```bash
   kubectl get nodes
   kubectl describe node <node-name>
   ```

3. **Check service endpoints**
   ```bash
   kubectl get endpoints radium-workflow
   ```

4. **Check network policies**
   ```bash
   kubectl get networkpolicy radium-workflow -o yaml
   ```

### Memory Issues (OOMKilled)

1. **Check memory usage**
   ```bash
   kubectl top pods -l app=radium-workflow
   ```

2. **Review memory limits**
   ```bash
   kubectl get deployment radium-workflow -o jsonpath='{.spec.template.spec.containers[0].resources}'
   ```

3. **Increase memory limit if needed**
   ```bash
   kubectl patch deployment radium-workflow -p '{"spec":{"template":{"spec":{"containers":[{"name":"radium-workflow","resources":{"limits":{"memory":"1Gi"}}}]}}}}'
   ```

4. **Investigate memory leaks**
   - Review recent changes
   - Check for workflows causing memory growth

## Maintenance Procedures

### Planned Maintenance

1. **Notify stakeholders**
2. **Scale down gracefully**
   ```bash
   kubectl scale deployment/radium-workflow --replicas=0
   ```
3. **Perform maintenance**
4. **Scale back up**
   ```bash
   kubectl scale deployment/radium-workflow --replicas=2
   ```
5. **Verify health**

### Log Rotation

Logs are handled by Kubernetes/container runtime. Ensure:
- Container logs are configured for rotation
- Log aggregation (e.g., Loki, Elasticsearch) is working

### Certificate Renewal

If using TLS termination:
```bash
# Check certificate expiry
kubectl get secret radium-workflow-tls -o jsonpath='{.data.tls\.crt}' | base64 -d | openssl x509 -noout -dates

# Renew certificates (cert-manager)
kubectl delete secret radium-workflow-tls
# cert-manager will auto-renew
```

## Contacts

| Role | Contact |
|------|---------|
| Service Owner | @team-radium |
| On-Call Engineer | PagerDuty: radium-workflow |
| Platform Team | @platform-team |

## Related Documentation

- [Troubleshooting Guide](./TROUBLESHOOTING.md)
- [Architecture Overview](../architecture/OVERVIEW.md)
- [API Documentation](../api/README.md)
