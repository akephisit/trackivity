# Trackivity - University Activity Tracking System

A comprehensive university activity tracking system built with Rust (Axum) backend and SvelteKit frontend.

## Features

### Backend (Rust + Axum)
- **REST API**: Full-featured REST API with proper error handling
- **PostgreSQL Database**: Robust data storage with sqlx migrations
- **Redis Sessions**: Secure session-based authentication (7-30 days expiry)
- **Multi-level Admin System**: Super Admin, Faculty Admin, Regular Admin
- **QR Code Integration**: Unique QR codes for each user
- **Real-time Updates**: Server-Sent Events (SSE) support

### Frontend (SvelteKit + TypeScript)
- **Modern UI**: Built with shadcn-svelte components
- **Responsive Design**: Works on desktop and mobile
- **Real-time Updates**: SSE client for live data
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

1. **Build and run with Docker Compose**
   ```bash
   docker-compose up --build
   ```

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