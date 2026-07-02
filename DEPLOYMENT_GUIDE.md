# Aura - Deployment Guide

Complete guide for deploying Aura platform to production environments.

## Table of Contents
1. [Local Development Setup](#local-development-setup)
2. [Production Deployment](#production-deployment)
3. [Docker Deployment](#docker-deployment)
4. [Cloud Deployment](#cloud-deployment)
5. [Troubleshooting](#troubleshooting)

---

## Local Development Setup

### Prerequisites
- Rust 1.70+ (install from https://rustup.rs/)
- Flutter 3.x (install from https://flutter.dev/docs/get-started/install)
- SQLite 3.x
- Git

### Step 1: Clone Repository
```bash
git clone https://github.com/foxyvio/Aura.git
cd Aura
```

### Step 2: Setup Backend

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source cargo environment
source "$HOME/.cargo/env"

# Build backend
cargo build --release

# Create .env file
cat > .env << EOF
DATABASE_URL=sqlite:aura.db
HOST=127.0.0.1
PORT=8080
JWT_SECRET=your-secret-key-change-this-in-production
EOF

# Run backend
cargo run --release
```

Backend will be available at `http://localhost:8080`

### Step 3: Setup Frontend

```bash
cd frontend

# Install Flutter (if not already installed)
git clone https://github.com/flutter/flutter.git -b stable
export PATH="$PATH:$(pwd)/flutter/bin"

# Enable web support
flutter config --enable-web

# Get dependencies
flutter pub get

# Run development server
flutter run -d web-server
```

Frontend will be available at `http://localhost:8000`

---

## Production Deployment

### Backend Deployment

#### 1. Build Release Binary
```bash
# Build optimized release binary
cargo build --release

# Binary location: target/release/aura_backend
# Size: ~50-80 MB (depending on dependencies)
```

#### 2. Prepare Production Environment
```bash
# Create production directory
mkdir -p /opt/aura
cd /opt/aura

# Copy binary
cp /path/to/Aura/target/release/aura_backend ./

# Create production .env
cat > .env << EOF
DATABASE_URL=sqlite:/opt/aura/data/aura.db
HOST=0.0.0.0
PORT=8080
JWT_SECRET=your-very-secret-key-here-min-32-chars
RUST_LOG=info
EOF

# Create data directory
mkdir -p data
chmod 755 data
```

#### 3. Setup Systemd Service (Linux)
```bash
# Create service file
sudo tee /etc/systemd/system/aura.service > /dev/null << EOF
[Unit]
Description=Aura Agent Marketplace Backend
After=network.target

[Service]
Type=simple
User=aura
WorkingDirectory=/opt/aura
ExecStart=/opt/aura/aura_backend
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Create aura user
sudo useradd -r -s /bin/false aura

# Set permissions
sudo chown -R aura:aura /opt/aura

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable aura
sudo systemctl start aura

# Check status
sudo systemctl status aura
```

#### 4. Setup Nginx Reverse Proxy
```bash
# Install Nginx
sudo apt-get install -y nginx

# Create Nginx config
sudo tee /etc/nginx/sites-available/aura > /dev/null << 'EOF'
upstream aura_backend {
    server 127.0.0.1:8080;
}

server {
    listen 80;
    server_name your-domain.com;

    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;

    # SSL certificates (use Let's Encrypt)
    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "SAMEORIGIN" always;

    # API proxy
    location /api/ {
        proxy_pass http://aura_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 90;
    }

    # Frontend static files
    location / {
        root /opt/aura/frontend/build/web;
        try_files $uri $uri/ /index.html;
    }
}
EOF

# Enable site
sudo ln -s /etc/nginx/sites-available/aura /etc/nginx/sites-enabled/

# Test Nginx config
sudo nginx -t

# Restart Nginx
sudo systemctl restart nginx
```

#### 5. Setup SSL Certificate (Let's Encrypt)
```bash
# Install Certbot
sudo apt-get install -y certbot python3-certbot-nginx

# Get certificate
sudo certbot certonly --nginx -d your-domain.com

# Auto-renewal
sudo systemctl enable certbot.timer
sudo systemctl start certbot.timer
```

### Frontend Deployment

#### 1. Build Web Assets
```bash
cd frontend

# Build production web
flutter build web --release

# Output: build/web/
```

#### 2. Deploy Static Files
```bash
# Copy to Nginx directory
sudo cp -r build/web/* /opt/aura/frontend/build/web/

# Set permissions
sudo chown -R www-data:www-data /opt/aura/frontend/build/web
```

---

## Docker Deployment

### Create Dockerfile
```dockerfile
# Multi-stage build for Rust backend
FROM rust:latest as rust_builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Flutter web build stage
FROM cirrusci/flutter:latest as flutter_builder
WORKDIR /app
COPY frontend ./frontend
RUN cd frontend && flutter build web --release

# Final runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    sqlite3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy backend binary
COPY --from=rust_builder /app/target/release/aura_backend /usr/local/bin/

# Copy frontend assets
COPY --from=flutter_builder /app/frontend/build/web /opt/aura/frontend/build/web

# Create app directory
WORKDIR /opt/aura

# Expose port
EXPOSE 8080

# Set environment
ENV DATABASE_URL=sqlite:/opt/aura/data/aura.db
ENV HOST=0.0.0.0
ENV PORT=8080
ENV RUST_LOG=info

# Create data directory
RUN mkdir -p /opt/aura/data

# Run backend
CMD ["aura_backend"]
```

### Build and Run Docker Image
```bash
# Build image
docker build -t aura:latest .

# Run container
docker run -d \
  -p 8080:8080 \
  -v aura_data:/opt/aura/data \
  -e JWT_SECRET="your-secret-key" \
  --name aura \
  aura:latest

# View logs
docker logs -f aura

# Stop container
docker stop aura
```

### Docker Compose
```yaml
# docker-compose.yml
version: '3.8'

services:
  aura:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - aura_data:/opt/aura/data
    environment:
      DATABASE_URL: sqlite:/opt/aura/data/aura.db
      HOST: 0.0.0.0
      PORT: 8080
      JWT_SECRET: your-secret-key-here
      RUST_LOG: info
    restart: always

  nginx:
    image: nginx:latest
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - aura
    restart: always

volumes:
  aura_data:
```

Run with:
```bash
docker-compose up -d
```

---

## Cloud Deployment

### AWS EC2 Deployment

#### 1. Launch EC2 Instance
```bash
# Use Ubuntu 22.04 LTS AMI
# Instance type: t3.medium or larger
# Storage: 20GB EBS
# Security Group: Allow 80, 443, 8080
```

#### 2. Connect and Setup
```bash
# SSH into instance
ssh -i your-key.pem ubuntu@your-instance-ip

# Update system
sudo apt-get update && sudo apt-get upgrade -y

# Install dependencies
sudo apt-get install -y \
  build-essential \
  curl \
  git \
  nginx \
  sqlite3 \
  certbot \
  python3-certbot-nginx

# Clone repository
git clone https://github.com/foxyvio/Aura.git
cd Aura

# Follow production deployment steps above
```

### Heroku Deployment

#### 1. Create Heroku App
```bash
# Install Heroku CLI
curl https://cli-assets.heroku.com/install.sh | sh

# Login
heroku login

# Create app
heroku create aura-app-name
```

#### 2. Create Procfile
```
web: ./target/release/aura_backend
```

#### 3. Deploy
```bash
# Add Heroku remote
heroku git:remote -a aura-app-name

# Deploy
git push heroku main

# View logs
heroku logs --tail
```

### DigitalOcean App Platform

#### 1. Create App
- Go to https://cloud.digitalocean.com/apps
- Click "Create App"
- Connect GitHub repository
- Select main branch

#### 2. Configure
```yaml
name: aura
services:
  - name: backend
    github:
      repo: foxyvio/Aura
      branch: main
    build_command: cargo build --release
    run_command: ./target/release/aura_backend
    http_port: 8080
    envs:
      - key: DATABASE_URL
        value: sqlite:/opt/aura/data/aura.db
      - key: JWT_SECRET
        value: ${JWT_SECRET}
```

---

## Monitoring & Maintenance

### Monitoring Backend
```bash
# Check service status
sudo systemctl status aura

# View logs
sudo journalctl -u aura -f

# Monitor resources
top
htop
```

### Database Maintenance
```bash
# Backup database
sqlite3 /opt/aura/data/aura.db ".backup /backup/aura_$(date +%Y%m%d).db"

# Check database integrity
sqlite3 /opt/aura/data/aura.db "PRAGMA integrity_check;"

# Optimize database
sqlite3 /opt/aura/data/aura.db "VACUUM;"
```

### Log Rotation
```bash
# Create logrotate config
sudo tee /etc/logrotate.d/aura > /dev/null << EOF
/var/log/aura/*.log {
    daily
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 aura aura
}
EOF
```

---

## Troubleshooting

### Backend Won't Start
```bash
# Check if port is in use
sudo lsof -i :8080

# Check logs
sudo journalctl -u aura -n 50

# Verify database
sqlite3 /opt/aura/data/aura.db ".tables"
```

### High Memory Usage
```bash
# Monitor memory
free -h

# Check process memory
ps aux | grep aura_backend

# Restart service
sudo systemctl restart aura
```

### Database Locked
```bash
# Check for stuck processes
lsof /opt/aura/data/aura.db

# Kill stuck process if necessary
kill -9 <PID>

# Restart service
sudo systemctl restart aura
```

### SSL Certificate Issues
```bash
# Check certificate expiry
sudo certbot certificates

# Renew certificate
sudo certbot renew --force-renewal

# Check Nginx SSL config
sudo nginx -t
```

---

## Performance Optimization

### Database Optimization
```sql
-- Add indexes for common queries
CREATE INDEX idx_agents_user_id ON agents(user_id);
CREATE INDEX idx_skills_agent_id ON skills(agent_id);
CREATE INDEX idx_transactions_seller ON transactions(seller_agent_id);
CREATE INDEX idx_transactions_buyer ON transactions(buyer_agent_id);
```

### Caching
```bash
# Add Redis for caching
docker run -d -p 6379:6379 redis:latest
```

### Load Balancing
```nginx
upstream aura_backend {
    server 127.0.0.1:8080;
    server 127.0.0.1:8081;
    server 127.0.0.1:8082;
}
```

---

## Security Checklist

- [ ] Change JWT_SECRET to a strong random value
- [ ] Enable HTTPS with valid SSL certificate
- [ ] Configure firewall to allow only necessary ports
- [ ] Restrict database access to localhost only
- [ ] Enable database backups
- [ ] Setup monitoring and alerting
- [ ] Configure rate limiting
- [ ] Enable CORS only for trusted domains
- [ ] Implement request logging
- [ ] Regular security updates

---

## Backup & Recovery

### Automated Backups
```bash
#!/bin/bash
# backup.sh
BACKUP_DIR="/backups/aura"
DB_FILE="/opt/aura/data/aura.db"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR
sqlite3 $DB_FILE ".backup $BACKUP_DIR/aura_$DATE.db"

# Keep only last 30 days
find $BACKUP_DIR -name "aura_*.db" -mtime +30 -delete
```

Add to crontab:
```bash
0 2 * * * /opt/aura/backup.sh
```

### Recovery
```bash
# Restore from backup
sqlite3 /opt/aura/data/aura.db ".restore /backups/aura/aura_20240701_020000.db"

# Restart service
sudo systemctl restart aura
```

---

For more information, visit: https://github.com/foxyvio/Aura

**Happy Deploying! 🚀**
