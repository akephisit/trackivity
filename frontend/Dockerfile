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

# Copy only production dependencies
COPY --from=builder /app/package.json /app/package-lock.json ./
RUN npm ci --omit=dev --no-audit --no-fund

# Copy the built application
COPY --from=builder /app/build ./build

ENV NODE_ENV=production
EXPOSE 3000

# Run the Node.js server
CMD ["node", "build"]
