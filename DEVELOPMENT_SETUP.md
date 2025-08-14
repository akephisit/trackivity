# Trackivity Development Setup Guide

## ภาพรวมของระบบ

Trackivity เป็นระบบติดตามกิจกรรมมหาวิทยาลัยที่ครบถ้วน ประกอบด้วย:

### Backend (Rust + Axum)
- **Framework**: Axum web framework
- **Database**: PostgreSQL with SQLx migrations
- **Session Store**: Redis with session-based authentication
- Real-time: (temporarily disabled)
- **Security**: Multi-level admin system (Super Admin, Faculty Admin, Regular Admin)

### Frontend (SvelteKit + TypeScript)
- **Framework**: SvelteKit 2.0
- **UI Components**: ShadcnUI-svelte
- **Authentication**: Session-based with auto-redirect
- Real-time: (temporarily disabled)
- **State Management**: Svelte stores

## การติดตั้งและเริ่มต้น

### 1. Requirements
```bash
# Backend
- Rust 1.70+
- PostgreSQL 15+
- Redis 7+

# Frontend
- Node.js 18+
- npm หรือ pnpm

# Docker (ทางเลือก)
- Docker 24+
- Docker Compose 2.20+
```

### 2. Environment Variables

#### Backend (.env)
```bash
# Database
DATABASE_URL=postgresql://postgres:password@localhost:5432/trackivity

# Redis
REDIS_URL=redis://localhost:6379

# Server
PORT=3000

# Security
SESSION_SECRET=your-very-secure-session-secret-here-change-in-production
SESSION_MAX_AGE=2592000  # 30 days
BCRYPT_COST=12

# CORS (development)
RUST_LOG=debug
```

#### Frontend (.env)
```bash
# API URLs
PUBLIC_API_URL=http://localhost:3000
VITE_API_URL=http://localhost:3000

# Development
NODE_ENV=development
```

### 3. การติดตั้งผ่าน Docker Compose (แนะนำ)

```bash
# Clone และเข้าไดเรกทอรี
cd /home/kruakemaths/github/trackivity

# สร้าง environment files
cp backend/.env.example backend/.env
cp frontend/.env.example frontend/.env

# เริ่มต้นบริการทั้งหมด
docker-compose up -d

# ตรวจสอบสถานะ
docker-compose logs -f
```

### 4. การติดตั้งแบบ Manual

#### Backend Setup
```bash
cd backend

# Install dependencies
cargo build

# สร้างฐานข้อมูล
createdb trackivity

# รัน migrations
cargo run --bin migrate

# เริ่ม backend server
cargo run
```

#### Frontend Setup
```bash
cd frontend

# Install dependencies
npm install

# สร้าง components และ types
npm run prepare

# เริ่ม development server
npm run dev
```

## การทดสอบระบบ

### 1. ตรวจสอบการเชื่อมต่อฐานข้อมูล

```bash
# ตรวจสอบ PostgreSQL
psql -h localhost -U postgres -d trackivity -c "SELECT COUNT(*) FROM users;"

# ตรวจสอบ Redis
redis-cli ping
redis-cli KEYS "session:*"
```

### 2. Default Admin Account

ระบบได้สร้างบัญชี Admin เริ่มต้นไว้แล้ว:

```
Email: admin@trackivity.local
Password: admin123!
Role: Super Admin
```

**⚠️ สำคัญ**: เปลี่ยนรหัสผ่านทันทีหลังจากล็อกอินครั้งแรก!

### 3. การทดสอบ Authentication Flow

#### 3.1 Student Login
```bash
# Test student registration
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "student_id": "64123456789",
    "email": "student@university.ac.th",
    "password": "password123",
    "first_name": "Test",
    "last_name": "Student"
  }'

# Test student login
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "student_id": "64123456789",
    "password": "password123"
  }'
```

#### 3.2 Admin Login
```bash
# Test admin login
curl -X POST http://localhost:3000/api/admin/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@trackivity.local",
    "password": "admin123!"
  }'
```

### 4. การทดสอบ Session Management

```bash
# Get current user (with session cookie)
curl -X GET http://localhost:3000/api/auth/me \
  -H "Cookie: session_id=YOUR_SESSION_ID"

# Get user sessions
curl -X GET http://localhost:3000/api/auth/sessions \
  -H "Cookie: session_id=YOUR_SESSION_ID"

# Test admin session management
curl -X GET http://localhost:3000/api/admin/sessions \
  -H "Cookie: session_id=ADMIN_SESSION_ID"
```

<!-- SSE testing temporarily disabled -->

## URLs และ Endpoints

### Frontend URLs
- **Homepage**: http://localhost:5173
- **Student Login**: http://localhost:5173/login
- **Student Registration**: http://localhost:5173/register
- **Admin Login**: http://localhost:5173/admin/login
- **Admin Dashboard**: http://localhost:5173/admin
- **Activities**: http://localhost:5173/activities

### Backend API Endpoints

#### Authentication
- `POST /api/auth/login` - Student login
- `POST /api/auth/register` - Student registration
- `POST /api/auth/logout` - Logout
- `GET /api/auth/me` - Get current user
- `GET /api/auth/sessions` - Get user sessions

#### Admin Authentication
- `POST /api/admin/auth/login` - Admin login
- `POST /api/admin/auth/logout` - Admin logout
- `GET /api/admin/auth/me` - Get admin user info

#### Admin Management
- `GET /api/admin/sessions` - Get all sessions (Super Admin)
- `DELETE /api/admin/sessions/:id` - Revoke session
- `POST /api/admin/sessions/cleanup` - Cleanup expired sessions

#### Activities
- `GET /api/activities` - List activities
- `POST /api/activities` - Create activity (Admin)
- `GET /api/activities/:id` - Get activity details
- `PUT /api/activities/:id` - Update activity (Admin)
- `DELETE /api/activities/:id` - Delete activity (Admin)
- `POST /api/activities/:id/participate` - Join activity
- `POST /api/activities/:id/scan` - QR scan attendance

<!-- SSE endpoints temporarily disabled -->

## Admin System

### Permission Levels

#### Super Admin
- จัดการระบบทั้งหมด
- จัดการ Faculty และ Department
- จัดการ Admin อื่นๆ
- ดู Session ทั้งหมด
- จัดการผู้ใช้ทั้งหมด

#### Faculty Admin
- จัดการเฉพาะ Faculty ของตนเอง
- จัดการนักเรียนใน Faculty
- สร้างและจัดการกิจกรรม
- ดูรายงานของ Faculty

#### Regular Admin
- สิทธิ์พื้นฐานในการจัดการ
- ช่วยงาน Faculty Admin
- สแกน QR Code

### Default Permissions

```rust
// Super Admin
ViewSystemReports, ManageAllFaculties, ManageUsers, 
ManageActivities, ManageAdmins, ManageSessions, ViewAllReports

// Faculty Admin
ManageFacultyUsers, ManageFacultyActivities, ViewFacultyReports, ScanQrCodes

// Regular Admin
ScanQrCodes, ViewAssignedActivities
```

## Database Schema

### Core Tables
- `users` - ข้อมูลผู้ใช้และนักเรียน
- `admin_roles` - บทบาทและสิทธิ์ Admin
- `faculties` - คณะ
- `departments` - ภาควิชา
- `activities` - กิจกรรม
- `participations` - การเข้าร่วมกิจกรรม
- `subscriptions` - การสมัครสมาชิก
- `sessions` - ข้อมูล Session (PostgreSQL)

### Session Storage
- Redis: Primary session storage
- PostgreSQL: Session metadata และ audit log

<!-- Real-time features temporarily disabled -->

## Troubleshooting

### Backend Issues
```bash
# ตรวจสอบ database connection
cargo run --bin check-db

# ตรวจสอบ Redis connection
cargo run --bin check-redis

# View logs
docker-compose logs backend -f
```

### Frontend Issues
```bash
# Clear cache และ rebuild
rm -rf node_modules .svelte-kit
npm install
npm run dev

# ตรวจสอบ API connection
curl http://localhost:3000/health
```

### Session Issues
```bash
# Clear all Redis sessions
redis-cli FLUSHDB

# ตรวจสอบ session ใน PostgreSQL
psql -d trackivity -c "SELECT * FROM sessions ORDER BY created_at DESC LIMIT 10;"
```

## Production Deployment

### Security Checklist
- [ ] เปลี่ยน `SESSION_SECRET`
- [ ] เปลี่ยนรหัสผ่าน default admin
- [ ] ตั้งค่า HTTPS
- [ ] จำกัด CORS origins
- [ ] ตั้งค่า rate limiting
- [ ] Backup database และ Redis
- [ ] Monitor session storage
- [ ] Log rotation

### Environment Variables (Production)
```bash
# Backend
DATABASE_URL=postgresql://user:pass@prod-db:5432/trackivity
REDIS_URL=redis://prod-redis:6379
SESSION_SECRET=very-secure-secret-64-chars-or-more
RUST_LOG=info

# Frontend
PUBLIC_API_URL=https://api.trackivity.com
VITE_API_URL=https://api.trackivity.com
```

## Support และ Maintenance

### Monitoring
- Session count และ cleanup
- Database performance
- Redis memory usage
<!-- SSE connection metrics temporarily removed -->
- API response times

### Regular Tasks
- Cleanup expired sessions
- Database vacuum และ analyze
- Log rotation
- Security updates
- Backup verification

---

**หมายเหตุ**: ระบบนี้พร้อมใช้งานแล้ว โปรดปฏิบัติตามคำแนะนำด้านความปลอดภัยสำหรับการใช้งานจริง
