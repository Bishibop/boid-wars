# Security Checklist

## Development

- [ ] SSL certificates are generated locally (not committed)
- [ ] No secrets in environment files
- [ ] Dependencies are audited regularly
- [ ] CORS headers configured properly

## WebTransport Security

- [ ] Rate limiting implemented
- [ ] Connection limits per IP
- [ ] Input validation on all messages
- [ ] Message size limits enforced

## Deployment

- [ ] Production SSL certificates from trusted CA
- [ ] Environment variables properly secured
- [ ] Health check endpoints don't expose sensitive data
- [ ] Logging doesn't include PII or secrets
- [ ] DDoS protection enabled

## Client Security

- [ ] Content Security Policy (CSP) headers
- [ ] XSS protection
- [ ] No client-side secrets
- [ ] Validate all server data

## Regular Audits

- [ ] Run `cargo audit` for Rust dependencies
- [ ] Run `npm audit` for JavaScript dependencies
- [ ] Review security headers
- [ ] Test rate limiting