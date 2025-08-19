# Trackivity - University Activity Tracking System

A comprehensive university activity tracking system built with Rust (Axum) backend and SvelteKit frontend.

## Features

### Backend (Rust + Axum)
- **REST API**: Full-featured REST API with proper error handling
- **PostgreSQL Database**: Robust data storage with sqlx migrations
- **Redis Sessions**: Secure session-based authentication (7-30 days expiry)
- **Multi-level Admin System**: Super Admin, Faculty Admin, Regular Admin
- **QR Code Integration**: Unique QR codes for each user
- Real-time Updates: (temporarily disabled)

### Frontend (SvelteKit + TypeScript)
- **Modern UI**: Built with shadcn-svelte components
- **Responsive Design**: Works on desktop and mobile
- Real-time Updates: (temporarily disabled)
- **Role-based Navigation**: Different interfaces for different user types
- **Thai Language Support**: Fully localized interface

### Database Schema
- **Users**: Students and administrators with QR secret keys
- **Faculties & Departments**: University organizational structure
- **Admin Roles**: Hierarchical permission system
- **Activities**: Event management with participation tracking
- **Participations**: Check-in/check-out system with status tracking
- **Subscriptions**: User subscription management with expiry tracking

## Quick Start

### Prerequisites
- Docker and Docker Compose
- Node.js 18+ (for frontend development)
- Rust 1.75+ (for backend development)

### Development Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd trackivity
   ```

2. **Start the database services**
   ```bash
   docker-compose up postgres redis -d
   ```

3. **Set up environment variables**
   ```bash
   # Backend environment
   cp backend/.env.example backend/.env
   # Edit backend/.env with your configuration
   
   # Frontend environment
   cp frontend/.env.example frontend/.env
   # Edit frontend/.env if needed
   ```

4. **Run database migrations**
   ```bash
   cargo install sqlx-cli
   cd backend
   sqlx migrate run
   ```

5. **Start the backend**
   ```bash
   cd backend
   cargo run
   ```

6. **Start the frontend** (in another terminal)
   ```bash
   cd frontend
   npm install
   npm run dev
   ```

7. **Access the application**
   - Frontend: http://localhost:5173
   - Backend API: http://localhost:3000
   - PostgreSQL: localhost:5432
   - Redis: localhost:6379

### Production Deployment

1. **Prepare env files**
   ```bash
   # Backend
   cp backend/.env.example backend/.env
   # Frontend
   cp frontend/.env.example frontend/.env
   ```

2. **Build and run with Docker Compose**
   ```bash
   docker-compose up --build -d
   ```

3. **Access services**
   - Frontend: http://localhost:5173
   - Backend API: http://localhost:3000
   - PostgreSQL: localhost:5432
   - Redis: localhost:6379

4. **Logs and troubleshooting**
   ```bash
   docker-compose logs -f backend
   docker-compose logs -f frontend
   ```

### SQLx offline cache (required for Docker builds)

SQLx query macros validate SQL at compile time. Our Dockerfile supports both offline and online builds and uses cargo-chef for fast caching.

Recommended (fast & reliable): generate and commit an offline cache:

1. Ensure Postgres is running and migrations applied:
   ```bash
   docker-compose up -d postgres
   export DATABASE_URL=postgresql://postgres:password@localhost:5432/trackivity
   cargo install sqlx-cli --no-default-features --features rustls,postgres
   cd backend
   sqlx database create || true
   sqlx migrate run
   ```
2. Prepare the cache file in `backend/sqlx-data.json`:
   ```bash
   cargo sqlx prepare -- --bin trackivity
   ```
3. Commit `backend/sqlx-data.json`. The Dockerfile will auto-detect it and build with `SQLX_OFFLINE=true`.

Online fallback (when you can't commit the cache):
- Build with a live DB URL available to the builder (e.g., DO Managed PG):
  ```bash
  docker build \
    --build-arg SQLX_OFFLINE=false \
    --build-arg DATABASE_URL='postgresql://<user>:<pass>@<host>:<port>/<db>?sslmode=require' \
    -t trackivity-backend ./backend
  ```
  On DigitalOcean App Platform, set these as Build-time env vars.

### Separate Images (run independently)

- Backend
  - Build: `docker build -t trackivity-backend ./backend`
  - Run: `docker run -d --name trackivity-backend -p 3000:3000 --env-file backend/.env trackivity-backend`
  - Required env: `DATABASE_URL`, `REDIS_URL`, `PORT` (optional; defaults to 3000), `SESSION_SECRET`, `RUST_LOG`.

- Frontend
  - Build (set API URL at build time):
    ```bash
    docker build -t trackivity-frontend \
      --build-arg PUBLIC_API_URL=https://api.your-domain.com \
      ./frontend
    ```
  - Run: `docker run -d --name trackivity-frontend -p 5173:5173 trackivity-frontend`
  - Note: Vite/SvelteKit env vars are baked at build time. Rebuild the image to change API URL.

## API Endpoints

### Authentication
- `POST /api/auth/login` - User login
- `POST /api/auth/register` - User registration  
- `POST /api/auth/logout` - User logout
- `GET /api/auth/me` - Get current user info

### Health Check
- `GET /api/health` - Service health check

## Technology Stack

### Backend
- **Rust** - Systems programming language
- **Axum** - Web framework
- **PostgreSQL** - Primary database
- **Redis** - Session storage and caching
- **SQLx** - Database toolkit
- **Tower** - Middleware and services
- **bcrypt** - Password hashing
- **UUID** - Unique identifiers

### Frontend  
- **SvelteKit** - Full-stack web framework
- **TypeScript** - Type-safe JavaScript
- **Tailwind CSS** - Utility-first CSS framework
- **shadcn-svelte** - UI component library
- **Zod** - Schema validation

### DevOps
- **Docker** - Containerization
- **Docker Compose** - Multi-container orchestration

## Project Structure

```
trackivity/
├── backend/               # Rust backend
│   ├── src/              # Rust backend source
│   │   ├── handlers/     # HTTP request handlers
│   │   ├── middleware/   # Authentication & session middleware
│   │   ├── models/       # Data models and types
│   │   ├── services/     # Business logic services
│   │   ├── routes/       # API route definitions
│   │   ├── utils/        # Utility functions
│   │   ├── config.rs     # Configuration management
│   │   ├── database.rs   # Database connection
│   │   └── main.rs       # Application entry point
│   ├── migrations/       # Database migrations
│   ├── Cargo.toml       # Rust dependencies
│   ├── Dockerfile       # Backend container image
│   ├── .env.example     # Backend environment template
│   └── .env             # Backend environment config
├── frontend/             # SvelteKit frontend
│   ├── src/
│   │   ├── lib/          # Shared components and utilities
│   │   ├── routes/       # SvelteKit routes
│   │   └── app.html      # HTML template
│   ├── package.json      # Frontend dependencies
│   ├── .env.example     # Frontend environment template
│   └── .env             # Frontend environment config
├── docker-compose.yml    # Development environment
└── README.md           # This file
```

## Development

### Backend Development
- The backend uses Rust with Axum framework
- Database migrations are managed with sqlx
- Session management uses Redis for scalability
- All API responses use JSON format

### Frontend Development  
- The frontend is built with SvelteKit and TypeScript
- UI components use shadcn-svelte design system
- API calls are handled through a centralized client
- State management uses Svelte stores

### Database Migrations
```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run pending migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

## Security Features

- **Session-based Authentication**: Secure session cookies with Redis storage
- **Password Hashing**: bcrypt with configurable cost
- **CORS Protection**: Configurable cross-origin resource sharing
- **Input Validation**: Server-side validation for all inputs
- **SQL Injection Prevention**: Parameterized queries with sqlx

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support, please open an issue on the GitHub repository or contact the development team.
