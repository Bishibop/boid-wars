# Hetzner WebTransport Deployment Proposal

## Executive Summary

Deploy Boid Wars to Hetzner Cloud with full WebTransport support for ultra-low latency multiplayer gameplay while maintaining traditional web hosting for the game's landing page and WASM client delivery. This proposal outlines a dual-service architecture that leverages Hetzner's excellent price-performance ratio and UDP support.

## Project Requirements

### Current Architecture
- **Game Server**: Rust + Bevy ECS + Lightyear 0.20 (WebSocket)
- **Web Serving**: Python proxy serves static files and forwards WebSocket
- **Static Content**: Landing page, WASM game client, game assets
- **Deployment**: Single Docker container on Fly.io

### Technical Goals
- Migrate from WebSocket to WebTransport for game traffic
- Maintain web presence for landing page and client delivery
- Achieve <50ms latency for EU players
- Support 10,000+ entities at stable 60 FPS

## Why Hetzner + WebTransport?

### Hetzner Cloud Advantages
- **Cost-effective**: CCX23 (4 dedicated vCPUs, 16GB RAM) at €21.50/month
- **Performance**: Dedicated vCPUs ensure consistent performance
- **UDP Support**: Full UDP/QUIC support for WebTransport (ports 1024-65535)
- **Free DNS**: Complete DNS management at no extra cost
- **EU Data Centers**: Nuremberg, Falkenstein (Germany), Helsinki (Finland)

### WebTransport Benefits
- **50-70% lower latency** compared to WebSocket
- **Better congestion control** with QUIC protocol
- **Multiplexed streams** without head-of-line blocking
- **Perfect for bullet-hell gameplay** requiring instant response

### Current Limitations to Address
- Hetzner's stateless firewall requires careful UDP configuration
- No UDP load balancing (must implement at application level)
- Limited to EU/US regions (no Asia-Pacific presence)

## Proposed Architecture

### Option 1: Nginx + Game Server (Recommended)

```
Internet Traffic
    ├── HTTP/HTTPS (TCP 80/443) → Nginx Container
    │   ├── Serves index.html (landing page)
    │   ├── Serves demo.html (game page)
    │   ├── Serves WASM/JS client files
    │   └── Serves game assets
    │
    └── WebTransport (UDP 443) → Game Server Container
        └── Handles all game traffic
```

**Benefits**:
- Clean separation of concerns
- Nginx optimized for static content
- Game server focused on WebTransport
- Easy to scale independently

### Option 2: Caddy Server (Simpler Alternative)

```
Internet Traffic → Caddy Server (TCP 80/443, UDP 443)
    ├── Automatic Let's Encrypt certificates
    ├── Built-in HTTP/3 support
    ├── Static file serving
    └── Reverse proxy to game server
```

**Benefits**:
- Single service to manage
- Automatic HTTPS certificates
- Built-in HTTP/3 support
- Simpler configuration

### Option 3: Hybrid CDN Approach (Future Enhancement)

```
Static Content → Cloudflare CDN → Users
Game Traffic → Direct to Hetzner → Game Server
```

**Benefits**:
- Global CDN for static assets
- DDoS protection
- Analytics and monitoring
- Reduced bandwidth costs

## Implementation Plan

### Phase 1: Infrastructure Setup (Week 1)

1. **Provision Hetzner Server**
   - CCX23 instance (4 vCPUs, 16GB RAM, 160GB NVMe)
   - Configure firewall rules:
     ```
     # Web traffic
     TCP 80 (HTTP) - Inbound
     TCP 443 (HTTPS) - Inbound
     
     # WebTransport
     UDP 443 (QUIC) - Inbound
     UDP 1024-65535 - Inbound (for QUIC connections)
     ```

2. **Domain and DNS Configuration**
   - Register domain (e.g., boidwars.com)
   - Configure Hetzner DNS:
     - A record: @ → server IP
     - A record: www → server IP
     - A record: game → server IP

### Phase 2: Application Updates (Week 2)

1. **Server Code Modifications**
   ```rust
   // server/src/main.rs
   fn create_webtransport_config() -> ServerConfig {
       let transport = ServerTransport::WebTransportServer {
           server_addr,
           certificate: load_certificate(),
           private_key: load_private_key(),
       };
       // ... rest of config
   }
   ```

2. **Client Updates**
   ```rust
   // bevy-client/src/lib.rs
   let transport = ClientTransport::WebTransportClient {
       client_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
       server_addr,
       #[cfg(target_family = "wasm")]
       certificate_digest: env!("CERT_DIGEST"),
   };
   ```

3. **Remove Python Proxy**
   - Delete `scripts/simple-http-ws-proxy.py`
   - Update Dockerfile to remove proxy

### Phase 3: Container Architecture (Week 3)

1. **Docker Compose Configuration**
   ```yaml
   version: '3.8'
   services:
     nginx:
       image: nginx:alpine
       ports:
         - "80:80"
         - "443:443"
       volumes:
         - ./static:/usr/share/nginx/html:ro
         - ./certs:/etc/nginx/certs:ro
         - ./nginx.conf:/etc/nginx/nginx.conf:ro
       depends_on:
         - game-server
     
     game-server:
       build: .
       ports:
         - "443:8080/udp"
       environment:
         - RUST_LOG=info
         - TRANSPORT_TYPE=webtransport
         - CERT_PATH=/certs/cert.pem
         - KEY_PATH=/certs/key.pem
       volumes:
         - ./certs:/certs:ro
   ```

2. **Nginx Configuration**
   - Serve static files with caching
   - Enable gzip compression
   - Configure HTTPS with Let's Encrypt
   - Add security headers

### Phase 4: Deployment and Testing (Week 4)

1. **Certificate Management**
   - Use Certbot for Let's Encrypt certificates
   - Automatic renewal via cron job
   - Share certificates between Nginx and game server

2. **Deployment Script**
   ```bash
   #!/bin/bash
   # Deploy to Hetzner
   docker-compose build
   docker-compose down
   docker-compose up -d
   
   # Health checks
   curl -I https://boidwars.com
   ./test-webtransport.sh
   ```

3. **Monitoring Setup**
   - Prometheus metrics from game server
   - Nginx access/error logs
   - Server resource monitoring
   - Uptime monitoring

## Cost Analysis

### Monthly Costs
- **Hetzner CCX23**: €21.50/month
- **Domain**: ~€0.83/month (€10/year)
- **Total**: €22.33/month (~$24/month)

### Comparison with Current (Fly.io)
- **Fly.io**: ~$25/month
- **Hetzner**: ~$24/month
- **Savings**: Minimal, but better performance

### Future Scaling Options
- **CCX33**: €42.50/month (8 vCPUs, 32GB RAM)
- **CCX43**: €84.00/month (16 vCPUs, 64GB RAM)
- **Multiple servers**: Implement custom UDP load balancing

## Risk Mitigation

### Technical Risks
1. **WebTransport Browser Support**
   - Risk: Safari doesn't support WebTransport
   - Mitigation: Implement WebSocket fallback
   
2. **Certificate Management**
   - Risk: Certificate expiry causing downtime
   - Mitigation: Automated renewal with monitoring

3. **UDP Firewall Complexity**
   - Risk: Misconfigured firewall blocking game traffic
   - Mitigation: Thorough testing, clear documentation

### Operational Risks
1. **Single Region Deployment**
   - Risk: High latency for non-EU players
   - Mitigation: Future multi-region deployment

2. **No Built-in Load Balancing**
   - Risk: Single server bottleneck
   - Mitigation: Application-level sharding

## Success Metrics

### Performance Targets
- **Latency**: <50ms for EU players
- **Frame Rate**: Stable 60 FPS with 10,000+ entities
- **Uptime**: 99.9% availability
- **Load Time**: <3 seconds for initial game load

### Monitoring Dashboard
- Real-time player count
- Average latency by region
- Server resource utilization
- WebTransport vs WebSocket usage ratio

## Migration Timeline

### Week 1: Infrastructure
- Provision Hetzner server
- Configure DNS and domains
- Set up basic monitoring

### Week 2: Application Updates
- Implement WebTransport in server
- Update client for WebTransport
- Remove Python proxy

### Week 3: Container Setup
- Configure Nginx for static serving
- Set up Docker Compose
- Implement certificate management

### Week 4: Production Launch
- Deploy to production
- Run performance tests
- Monitor and optimize

## Conclusion

This architecture provides a clean, scalable solution for deploying Boid Wars with WebTransport support. By separating static content serving from game traffic, we achieve optimal performance for both web delivery and real-time gameplay. The use of Hetzner's infrastructure provides excellent value while maintaining the performance standards required for a competitive multiplayer game.

The migration path is straightforward, with clear phases that can be tested independently. The total cost remains comparable to current hosting while providing significant performance improvements through WebTransport's lower latency and better congestion control.