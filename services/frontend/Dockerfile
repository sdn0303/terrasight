# -----------------------------------------------------------------------
# Stage 1: deps — install dependencies (cached separately from source)
# -----------------------------------------------------------------------
FROM node:22-slim AS deps

WORKDIR /app

COPY package.json pnpm-lock.yaml ./

RUN corepack enable pnpm && \
    pnpm install --frozen-lockfile

# -----------------------------------------------------------------------
# Stage 2: builder — build the Vite SPA
# -----------------------------------------------------------------------
FROM node:22-slim AS builder

WORKDIR /app

COPY --from=deps /app/node_modules ./node_modules
COPY . .

# Create empty public dirs that Vite expects (populated at runtime via volumes)
RUN mkdir -p public/data/fgb public/wasm

# Vite env vars are baked into the bundle at build time.
ARG VITE_API_URL=""
ARG VITE_MAPBOX_TOKEN=""
ENV VITE_API_URL=${VITE_API_URL}
ENV VITE_MAPBOX_TOKEN=${VITE_MAPBOX_TOKEN}

RUN corepack enable pnpm && \
    pnpm build

# -----------------------------------------------------------------------
# Stage 3: runner — serve static files with nginx
# -----------------------------------------------------------------------
FROM nginx:1.27-alpine AS runner

LABEL maintainer="terrasight"
LABEL description="Terrasight frontend — Vite SPA served by nginx"
LABEL version="0.1.0"

# Remove default nginx config
RUN rm /etc/nginx/conf.d/default.conf

# Custom nginx config for SPA routing + API reverse proxy
COPY <<'EOF' /etc/nginx/conf.d/terrasight.conf
server {
    listen 3000;
    server_name _;

    root /usr/share/nginx/html;
    index index.html;

    # Pre-compressed static files: serve .gz directly when available (zero CPU)
    gzip_static on;

    # Dynamic gzip fallback for non-pre-compressed responses
    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml image/svg+xml application/wasm application/octet-stream;
    gzip_min_length 256;

    # Static assets with long cache
    location /assets/ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # FlatGeobuf data files (pre-compressed .gz available)
    location /data/fgb/ {
        gzip_static on;
        expires 7d;
        add_header Cache-Control "public";
    }

    # WASM files (pre-compressed .gz available)
    location /wasm/ {
        gzip_static on;
        expires 7d;
        add_header Cache-Control "public";
        add_header Cross-Origin-Embedder-Policy "require-corp";
        add_header Cross-Origin-Opener-Policy "same-origin";
    }

    # API reverse proxy to backend
    location /api/ {
        proxy_pass http://backend:8000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # SPA fallback — all non-file routes serve index.html
    location / {
        try_files $uri $uri/ /index.html;
    }
}
EOF

# Copy build output
COPY --from=builder /app/dist /usr/share/nginx/html

# nginx runs as non-root by default in the alpine image (nginx user)
# Override the default port (80 → 3000) via config above

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD ["wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3000/"]
