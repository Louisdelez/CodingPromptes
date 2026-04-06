# === Stage 1: Build Rust API server ===
FROM rust:1.83-slim-bookworm AS rust-build

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY inkwell-api-server/ ./
RUN cargo build --release

# === Stage 2: Build Web frontend ===
FROM node:22-alpine AS web-build

WORKDIR /app
COPY inkwell/package.json inkwell/package-lock.json ./
RUN npm ci
COPY inkwell/ ./
RUN npm run build

# === Stage 3: Runtime ===
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y nginx ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy API server binary
COPY --from=rust-build /build/target/release/inkwell-api-server /usr/local/bin/inkwell-api-server

# Copy web frontend
COPY --from=web-build /app/dist /usr/share/nginx/html

# Nginx config: serve frontend + proxy API to backend
RUN cat > /etc/nginx/conf.d/default.conf <<'NGINX'
server {
    listen 80;
    server_name _;
    root /usr/share/nginx/html;
    index index.html;

    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml text/javascript image/svg+xml;
    gzip_min_length 256;

    location /assets/ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Proxy API + LLM to the Rust backend
    location /api/ {
        proxy_pass http://127.0.0.1:8910;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /v1/ {
        proxy_pass http://127.0.0.1:8910;
        proxy_set_header Host $host;
    }

    location /health {
        proxy_pass http://127.0.0.1:8910;
    }

    # SPA fallback
    location / {
        try_files $uri $uri/ /index.html;
    }
}
NGINX

# Startup script: launch API server + nginx
RUN cat > /start.sh <<'EOF'
#!/bin/sh
echo "Starting Inkwell API Server..."
inkwell-api-server &
sleep 1
echo "Starting Nginx..."
nginx -g "daemon off;"
EOF
RUN chmod +x /start.sh

# Data volume for SQLite
VOLUME /root/.local/share/inkwell-server

EXPOSE 80

ENV PORT=8910
ENV JWT_SECRET=change-me-in-production
ENV OLLAMA_URL=http://host.docker.internal:11434

CMD ["/start.sh"]
