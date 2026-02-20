# Security Vulnerabilities and Remediation

This document tracks known security vulnerabilities and their remediation status.

## Current Vulnerabilities

### Rust Dependencies

#### RUSTSEC-2025-0009: ring 0.17.9 (Critical)
- **Status**: Temporarily ignored (awaiting upstream fix)
- **Severity**: Critical
- **Description**: Some AES functions may panic when overflow checking is enabled
- **Affected Package**: `ring 0.17.9`
- **Required Version**: `>=0.17.12`
- **Dependency Path**: `ring` → `rustls-webpki 0.103.8` → `rustls 0.23.35` → `hyper-rustls` → `reqwest`
- **Issue**: The vulnerability is in a transitive dependency. `rustls-webpki 0.103.8` depends on `ring 0.17.9`, which needs to be upgraded to `>=0.17.12`. However, this requires an upstream update to `rustls-webpki`.
- **Tracking**: https://github.com/rustls/webpki/issues
- **Action**: Monitoring for `rustls-webpki` release that uses `ring >=0.17.12`
- **Temporary Exception**: Added to `config/deny.toml` with reason: "Waiting for rustls-webpki to update ring dependency"

#### Unmaintained Crates (Warnings)
The following crates are marked as unmaintained but are still functional:

1. **fxhash 0.2.1** (RUSTSEC-2025-0057)
   - Used by: `inquire` → `radium-cli`
   - Status: Monitoring for replacement
   - Alternative: Consider using `ahash` or `hashbrown` with custom hasher

2. **number_prefix 0.4.0** (RUSTSEC-2025-0119)
   - Used by: `indicatif` → `radium-core`
   - Status: Monitoring for replacement
   - Alternative: Consider using `byte-unit` or implementing custom formatting

3. **paste 1.0.15** (RUSTSEC-2024-0436)
   - Used by: `ratatui` → `radium-tui`, `radium-core`
   - Status: Monitoring for replacement
   - Alternative: Consider using `paste` crate alternatives or `proc-macro2` directly

4. **proc-macro-error 1.0.4** (RUSTSEC-2024-0370)
   - Used by: `tabled_derive` → `tabled` → `radium-cli`
   - Status: Monitoring for replacement
   - Alternative: Consider using `syn` error handling directly

5. **yaml-rust 0.4.5** (RUSTSEC-2024-0320)
   - Used by: `syntect` → `radium-tui`, `radium-core`
   - Status: Monitoring for replacement
   - Alternative: Consider using `serde_yaml` or `yaml` crate

**Note**: These are warnings, not critical vulnerabilities. They indicate the crates are no longer actively maintained, which may lead to unpatched security issues in the future. We should plan to replace them when feasible.

### JavaScript/TypeScript Dependencies

#### Next.js Vulnerabilities (via @nx/next)
- **Status**: Partially addressed - requires @nx/next update
- **Affected Package**: `next@16.0.6` (via `@nx/next`)
- **Vulnerabilities**:
  1. **GHSA-9qr9-h5gf-34mp** (Critical): RCE in React flight protocol
  2. **GHSA-mwv6-3258-q52c** (High): Denial of Service with Server Components
  3. **GHSA-w37m-7fhw-fmv9** (Moderate): Server Actions Source Code Exposure
- **Required Version**: `>=16.0.7`
- **Current Version**: `16.0.6` (pulled in by `@nx/next@22.2.1`)
- **Action Taken**: 
  - Added `next@16.0.10` as dev dependency
  - Added package override in `package.json` to force `next >=16.0.7`
- **Issue**: `@nx/next` has a peer dependency on `next <17.0.0` and is resolving to `16.0.6`. The override doesn't affect peer dependencies.
- **Next Steps**: 
  - Monitor for `@nx/next` update that supports `next >=16.0.7`
  - Consider using `resolutions` field if Bun supports it (currently using `overrides`)
  - Alternative: Wait for Nx to update their Next.js peer dependency range

## Remediation Strategy

### Immediate Actions
1. ✅ Fixed `cargo-deny` license configuration
2. ✅ Added temporary exception for RUSTSEC-2025-0009 with tracking
3. ✅ Set up GitHub Dependabot for automated vulnerability scanning
4. ✅ Added package override for Next.js vulnerability
5. ⏳ Run `bun install` to apply Next.js override

### Ongoing Monitoring
1. Monitor `rustls-webpki` releases for `ring >=0.17.12` support
2. Review Dependabot pull requests weekly
3. Plan replacements for unmaintained crates
4. Run `cargo audit` and `bun audit` regularly

### Long-term Actions
1. Replace unmaintained crates with actively maintained alternatives
2. Consider using `cargo-deny` in CI/CD pipeline
3. Set up automated security scanning in GitHub Actions
4. Document security update procedures

## Security Scanning

### Manual Scanning
```bash
# Rust dependencies
cargo audit
cargo deny check --config config/deny.toml

# JavaScript/TypeScript dependencies
bun audit
```

### Automated Scanning
- **Dependabot**: Configured in `.github/dependabot.yml`
  - Weekly scans for Rust (Cargo) and JavaScript (npm) dependencies
  - Automatic pull request creation for security updates

## References
- [RustSec Advisory Database](https://rustsec.org/advisories/)
- [GitHub Security Advisories](https://github.com/advisories)
- [cargo-audit](https://github.com/rustsec/cargo-audit)
- [cargo-deny](https://github.com/embarkstudios/cargo-deny)
