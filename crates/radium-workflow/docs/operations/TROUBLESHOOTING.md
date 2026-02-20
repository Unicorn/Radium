# Radium Workflow Compiler - Troubleshooting Guide

## Quick Diagnostics

### Service Status Check

```bash
# 1. Check pod status
kubectl get pods -l app=radium-workflow

# 2. Check recent events
kubectl get events --sort-by='.lastTimestamp' | grep radium-workflow | tail -10

# 3. Check logs for errors
kubectl logs -l app=radium-workflow --tail=50 | grep -i error

# 4. Check metrics endpoint
kubectl exec -it $(kubectl get pod -l app=radium-workflow -o jsonpath='{.items[0].metadata.name}') -- \
  wget -qO- http://localhost:3000/metrics | head -30
```

## Common Issues and Solutions

### Issue: Compilation Timeout

**Symptoms:**
- Compilations taking > 30 seconds
- Timeout errors in logs
- High latency metrics

**Diagnosis:**
```bash
# Check workflow size in logs
kubectl logs -l app=radium-workflow | grep "components" | tail -10

# Check CPU usage
kubectl top pods -l app=radium-workflow
```

**Solutions:**
1. **Large workflow**: Workflow has too many components
   - Advise user to split into smaller workflows
   - Consider using child workflows

2. **Resource starvation**: Pod needs more CPU
   ```bash
   kubectl patch deployment radium-workflow -p \
     '{"spec":{"template":{"spec":{"containers":[{"name":"radium-workflow","resources":{"limits":{"cpu":"2000m"}}}]}}}}'
   ```

3. **Inefficient expressions**: Complex expressions causing slow compilation
   - Review workflow expressions for optimization opportunities

### Issue: Validation Errors

**Symptoms:**
- `ValidationFailed` errors in logs
- HTTP 400 responses

**Diagnosis:**
```bash
# Get validation error details
kubectl logs -l app=radium-workflow | grep "validation" | tail -20
```

**Common Causes:**
1. **Cycle in workflow graph**
   - Error: "Workflow contains a cycle"
   - Solution: User must restructure workflow to remove circular dependencies

2. **Missing component connections**
   - Error: "Component has no incoming connections"
   - Solution: User must connect all components

3. **Invalid expression syntax**
   - Error: "Expression parse error"
   - Solution: User must fix expression syntax

4. **Type mismatch**
   - Error: "Type mismatch in expression"
   - Solution: User must ensure type compatibility

### Issue: Rate Limiting

**Symptoms:**
- HTTP 429 responses
- `rate_limit_exceeded` events in audit log

**Diagnosis:**
```bash
# Check rate limit metrics
kubectl exec -it $(kubectl get pod -l app=radium-workflow -o jsonpath='{.items[0].metadata.name}') -- \
  wget -qO- http://localhost:3000/metrics | grep rate_limit

# Check which clients are hitting limits
kubectl logs -l app=radium-workflow | grep "rate_limit" | awk '{print $NF}' | sort | uniq -c
```

**Solutions:**
1. **Legitimate high traffic**: Increase limits
   ```bash
   kubectl set env deployment/radium-workflow \
     RATE_LIMIT_REQUESTS=200 \
     RATE_LIMIT_WINDOW_SECS=60
   ```

2. **Abusive client**: Block specific client
   - Add client to blocklist
   - Review security policies

### Issue: Memory Pressure (OOMKilled)

**Symptoms:**
- Pods restarting with OOMKilled
- Memory metrics near limits

**Diagnosis:**
```bash
# Check memory usage
kubectl top pods -l app=radium-workflow

# Check for OOMKilled events
kubectl describe pods -l app=radium-workflow | grep -A5 "Last State:"

# Check memory metrics over time (if Prometheus available)
# query: container_memory_usage_bytes{pod=~"radium-workflow.*"}
```

**Solutions:**
1. **Increase memory limit**
   ```bash
   kubectl patch deployment radium-workflow -p \
     '{"spec":{"template":{"spec":{"containers":[{"name":"radium-workflow","resources":{"limits":{"memory":"1Gi"}}}]}}}}'
   ```

2. **Investigate memory growth**
   - Check for large workflows being processed
   - Review cache size configuration
   - Look for potential memory leaks

3. **Reduce cache size**
   ```bash
   kubectl set env deployment/radium-workflow CACHE_MAX_ENTRIES=500
   ```

### Issue: Pod Not Starting

**Symptoms:**
- Pod stuck in `Pending` or `CrashLoopBackOff`

**Diagnosis:**
```bash
# Check pod events
kubectl describe pod $(kubectl get pod -l app=radium-workflow -o jsonpath='{.items[0].metadata.name}')

# Check container logs
kubectl logs $(kubectl get pod -l app=radium-workflow -o jsonpath='{.items[0].metadata.name}') --previous
```

**Common Causes:**
1. **Image pull failure**
   - Check image exists in registry
   - Verify pull secrets

2. **Resource constraints**
   - Check node resources
   - Review resource requests

3. **Configuration error**
   - Check ConfigMap values
   - Verify environment variables

4. **Health check failure**
   - Review liveness/readiness probe configuration
   - Check application startup time

### Issue: High Error Rate

**Symptoms:**
- Increased `compilation_errors_total` metric
- Error logs increasing

**Diagnosis:**
```bash
# Categorize errors
kubectl logs -l app=radium-workflow --tail=500 | grep -i error | \
  awk -F'error' '{print $2}' | cut -c1-50 | sort | uniq -c | sort -rn

# Check error rate over time (if Prometheus available)
# query: rate(compilation_errors_total[5m])
```

**Solutions:**
1. **User input errors**: Normal - validation working as expected
2. **System errors**: Investigate infrastructure issues
3. **Code bugs**: Check recent deployments, consider rollback

### Issue: Connection Refused

**Symptoms:**
- Cannot connect to service
- HTTP connection errors

**Diagnosis:**
```bash
# Check service endpoints
kubectl get endpoints radium-workflow

# Check service definition
kubectl get svc radium-workflow -o yaml

# Test internal connectivity
kubectl run test-pod --rm -it --image=alpine -- wget -qO- http://radium-workflow/health/live
```

**Solutions:**
1. **No endpoints**: Pods not ready
   - Check pod health
   - Review readiness probe

2. **Network policy blocking**
   ```bash
   kubectl get networkpolicy radium-workflow -o yaml
   ```

3. **Service misconfiguration**
   - Verify selector matches pod labels
   - Check port configuration

### Issue: Slow Startup

**Symptoms:**
- Pod takes long time to become ready
- Readiness probe failures during startup

**Diagnosis:**
```bash
# Check startup timeline
kubectl describe pod $(kubectl get pod -l app=radium-workflow -o jsonpath='{.items[0].metadata.name}') | grep -A20 "Events:"

# Check startup logs
kubectl logs $(kubectl get pod -l app=radium-workflow -o jsonpath='{.items[0].metadata.name}') | head -50
```

**Solutions:**
1. **Increase startup probe timeout**
   ```yaml
   startupProbe:
     initialDelaySeconds: 10
     periodSeconds: 5
     failureThreshold: 30
   ```

2. **Investigate slow initialization**
   - Check cache warming
   - Review startup dependencies

## Diagnostic Commands Reference

### Log Analysis

```bash
# Recent errors
kubectl logs -l app=radium-workflow --tail=100 | grep -i error

# Compilation requests
kubectl logs -l app=radium-workflow | grep "compilation_requested"

# Rate limit events
kubectl logs -l app=radium-workflow | grep "rate_limit"

# Security events
kubectl logs -l app=radium-workflow | grep -E "(dangerous_pattern|security_alert)"
```

### Performance Analysis

```bash
# CPU/Memory usage
kubectl top pods -l app=radium-workflow

# Request latency (from metrics)
kubectl exec -it $(kubectl get pod -l app=radium-workflow -o jsonpath='{.items[0].metadata.name}') -- \
  wget -qO- http://localhost:3000/metrics | grep compilation_duration

# Cache stats
kubectl exec -it $(kubectl get pod -l app=radium-workflow -o jsonpath='{.items[0].metadata.name}') -- \
  wget -qO- http://localhost:3000/metrics | grep cache
```

### Network Diagnostics

```bash
# DNS resolution
kubectl run test --rm -it --image=alpine -- nslookup radium-workflow

# Port connectivity
kubectl run test --rm -it --image=alpine -- nc -zv radium-workflow 80

# Full HTTP test
kubectl run test --rm -it --image=alpine -- wget -qO- http://radium-workflow/health/live
```

## Escalation Path

1. **L1 Support**: Check this guide, basic diagnostics
2. **L2 Support**: Deep dive into logs/metrics, configuration review
3. **Development Team**: Code-level investigation, bug fixes
4. **Platform Team**: Infrastructure issues, Kubernetes problems

## Related Resources

- [Operations Runbook](./RUNBOOK.md)
- [Architecture Documentation](../architecture/OVERVIEW.md)
- [API Reference](../api/README.md)
