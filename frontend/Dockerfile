# Stage 1: Build
FROM node:20-alpine AS builder
WORKDIR /app

# Install dependencies
COPY package.json package-lock.json ./
RUN npm ci --no-audit --no-fund

# Copy source and build
COPY . .
ENV NODE_ENV=production

# Accept API URL at build time (baked into bundle)
ARG PUBLIC_API_URL
ENV PUBLIC_API_URL=${PUBLIC_API_URL}

RUN npm run build

# Stage 2: Runtime
FROM node:20-alpine AS runtime
WORKDIR /app

# Copy only what's needed to run preview server
COPY --from=builder /app/package.json /app/package-lock.json ./
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/.svelte-kit ./.svelte-kit
COPY --from=builder /app/build ./build
COPY --from=builder /app/src ./src
COPY --from=builder /app/svelte.config.js ./svelte.config.js
COPY --from=builder /app/vite.config.ts ./vite.config.ts

ENV NODE_ENV=production
EXPOSE 5173

# Use preview to serve built app
CMD ["npm", "run", "preview", "--", "--host", "0.0.0.0", "--port", "5173"]
