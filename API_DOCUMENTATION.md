# Trackivity API Documentation

## Overview

Trackivity API เป็น REST API ที่ใช้ session-based authentication และ Server-Sent Events สำหรับ real-time updates

**Base URL**: `http://localhost:3000`  
**Authentication**: Session cookies หรือ `X-Session-ID` header  
**Content Type**: `application/json`

---

## Authentication

### Student Authentication

#### Register Student
```http
POST /api/auth/register
```

**Request Body:**
```json
{
  "student_id": "64123456789",
  "email": "student@university.ac.th", 
  "password": "securepassword",
  "first_name": "John",
  "last_name": "Doe",
  "department_id": "uuid-optional"
}
```

**Response:**
```json
{
  "success": true,
  "user_id": "uuid",
  "message": "User registered successfully"
}
```

#### Student Login
```http
POST /api/auth/login
```

**Request Body:**
```json
{
  "student_id": "64123456789",
  "password": "securepassword",
  "remember_me": false
}
```

**Response:**
```json
{
  "success": true,
  "session": {
    "session_id": "session-uuid",
    "user": {
      "user_id": "uuid",
      "student_id": "64123456789",
      "email": "student@university.ac.th",
      "first_name": "John",
      "last_name": "Doe",
      "department_id": null,
      "admin_role": null,
      "permissions": ["ViewProfile", "UpdateProfile"],
      "faculty_id": null,
      "session_id": "session-uuid"
    },
    "expires_at": "2025-02-04T10:30:00Z"
  },
  "message": "Login successful"
}
```

#### Get Current User
```http
GET /api/auth/me
Cookie: session_id=your-session-id
```

**Response:**
```json
{
  "user_id": "uuid",
  "student_id": "64123456789", 
  "email": "student@university.ac.th",
  "first_name": "John",
  "last_name": "Doe",
  "department_id": null,
  "admin_role": null,
  "permissions": ["ViewProfile", "UpdateProfile"],
  "faculty_id": null,
  "session_id": "session-uuid"
}
```

#### Logout
```http
POST /api/auth/logout
Cookie: session_id=your-session-id
```

**Response:**
```json
{
  "success": true,
  "message": "Logged out successfully"
}
```

---

### Admin Authentication

#### Admin Login
```http
POST /api/admin/auth/login
```

**Request Body:**
```json
{
  "email": "admin@trackivity.local",
  "password": "admin123!",
  "remember_me": false
}
```

**Response:**
```json
{
  "success": true,
  "session": {
    "session_id": "admin-session-uuid",
    "user": {
      "user_id": "uuid",
      "student_id": "ADMIN001",
      "email": "admin@trackivity.local",
      "first_name": "System",
      "last_name": "Administrator",
      "admin_role": {
        "id": "uuid",
        "user_id": "uuid",
        "admin_level": "SuperAdmin",
        "faculty_id": null,
        "permissions": [
          "ViewSystemReports",
          "ManageAllFaculties",
          "ManageUsers",
          "ManageActivities",
          "ManageAdmins",
          "ManageSessions",
          "ViewAllReports"
        ]
      },
      "permissions": ["ViewSystemReports", "ManageAllFaculties", "..."],
      "faculty_id": null,
      "session_id": "admin-session-uuid"
    },
    "expires_at": "2025-02-04T10:30:00Z"
  },
  "message": "Admin login successful"
}
```

#### Get Admin Info
```http
GET /api/admin/auth/me
Cookie: session_id=admin-session-id
```

**Response:**
```json
{
  "user": {
    "user_id": "uuid",
    "email": "admin@trackivity.local",
    "admin_role": {
      "admin_level": "SuperAdmin",
      "permissions": ["ViewSystemReports", "..."]
    }
  },
  "admin_role": {
    "id": "uuid",
    "admin_level": "SuperAdmin",
    "faculty_id": null,
    "permissions": ["ViewSystemReports", "..."]
  },
  "permissions": ["ViewSystemReports", "ManageAllFaculties", "..."]
}
```

---

## Session Management

### Get User Sessions
```http
GET /api/auth/sessions
Cookie: session_id=your-session-id
```

**Response:**
```json
{
  "sessions": [
    {
      "session_id": "session-uuid",
      "device_info": {
        "browser": "Chrome",
        "os": "Windows",
        "device_type": "desktop"
      },
      "ip_address": "192.168.1.100",
      "user_agent": "Mozilla/5.0...",
      "created_at": "2025-01-05T08:00:00Z",
      "last_accessed": "2025-01-05T10:30:00Z",
      "expires_at": "2025-02-04T08:00:00Z"
    }
  ],
  "active_count": 1
}
```

### Revoke My Session
```http
DELETE /api/auth/sessions/{session_id}
Cookie: session_id=your-session-id
```

**Response:**
```json
{
  "success": true,
  "message": "Session revoked successfully"
}
```

### Extend Session
```http
POST /api/auth/sessions/extend
Cookie: session_id=your-session-id
```

**Request Body:**
```json
{
  "hours": 24
}
```

**Response:**
```json
{
  "success": true,
  "message": "Session extended successfully",
  "expires_at": "2025-02-05T10:30:00Z"
}
```

---

## Admin Session Management

### Get All Sessions (Super Admin)
```http
GET /api/admin/sessions?limit=50
Cookie: session_id=admin-session-id
```

**Response:**
```json
{
  "sessions": [
    {
      "session_id": "session-uuid",
      "user_id": "uuid",
      "user_name": "John Doe",
      "admin_level": null,
      "faculty_name": null,
      "device_info": {
        "browser": "Chrome",
        "os": "Windows"
      },
      "ip_address": "192.168.1.100",
      "user_agent": "Mozilla/5.0...",
      "created_at": "2025-01-05T08:00:00Z",
      "last_accessed": "2025-01-05T10:30:00Z",
      "expires_at": "2025-02-04T08:00:00Z"
    }
  ],
  "total_count": 25
}
```

### Revoke Session (Admin)
```http
DELETE /api/admin/sessions/{session_id}
Cookie: session_id=admin-session-id
```

**Request Body:**
```json
{
  "reason": "Security policy violation"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Session revoked successfully"
}
```

### Revoke All User Sessions (Admin)
```http
DELETE /api/admin/users/{user_id}/sessions
Cookie: session_id=admin-session-id
```

**Request Body:**
```json
{
  "reason": "Account security review"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Revoked 3 sessions",
  "revoked_count": 3
}
```

---

## Activities

### List Activities
```http
GET /api/activities?limit=20&offset=0&status=published
Cookie: session_id=your-session-id
```

**Query Parameters:**
- `limit`: Number of activities (default: 20)
- `offset`: Pagination offset (default: 0)
- `status`: Filter by status (draft, published, ongoing, completed, cancelled)
- `faculty_id`: Filter by faculty
- `start_date`: Filter by start date (ISO 8601)
- `end_date`: Filter by end date (ISO 8601)

**Response:**
```json
{
  "activities": [
    {
      "id": "uuid",
      "title": "Programming Workshop",
      "description": "Learn modern web development",
      "location": "Computer Lab 1",
      "start_time": "2025-01-10T09:00:00Z",
      "end_time": "2025-01-10T17:00:00Z",
      "max_participants": 30,
      "current_participants": 15,
      "status": "published",
      "faculty_id": "uuid",
      "department_id": "uuid",
      "created_by": "uuid",
      "created_at": "2025-01-05T10:00:00Z"
    }
  ],
  "total_count": 50,
  "has_more": true
}
```

### Get Activity Details
```http
GET /api/activities/{activity_id}
Cookie: session_id=your-session-id
```

**Response:**
```json
{
  "id": "uuid",
  "title": "Programming Workshop",
  "description": "Learn modern web development with React and Node.js",
  "location": "Computer Lab 1, Building A",
  "start_time": "2025-01-10T09:00:00Z",
  "end_time": "2025-01-10T17:00:00Z",
  "max_participants": 30,
  "current_participants": 15,
  "status": "published",
  "faculty": {
    "id": "uuid",
    "name": "Computer Science",
    "code": "CS"
  },
  "department": {
    "id": "uuid", 
    "name": "Software Engineering",
    "code": "SE"
  },
  "created_by": {
    "id": "uuid",
    "name": "Dr. Jane Smith"
  },
  "is_registered": false,
  "registration_deadline": "2025-01-09T23:59:59Z",
  "created_at": "2025-01-05T10:00:00Z",
  "updated_at": "2025-01-05T10:00:00Z"
}
```

### Create Activity (Admin)
```http
POST /api/activities
Cookie: session_id=admin-session-id
```

**Request Body:**
```json
{
  "title": "Programming Workshop",
  "description": "Learn modern web development",
  "location": "Computer Lab 1",
  "start_time": "2025-01-10T09:00:00Z",
  "end_time": "2025-01-10T17:00:00Z",
  "max_participants": 30,
  "faculty_id": "uuid",
  "department_id": "uuid"
}
```

**Response:**
```json
{
  "success": true,
  "activity": {
    "id": "uuid",
    "title": "Programming Workshop",
    "status": "draft"
  },
  "message": "Activity created successfully"
}
```

### Join Activity
```http
POST /api/activities/{activity_id}/participate
Cookie: session_id=your-session-id
```

**Response:**
```json
{
  "success": true,
  "participation": {
    "id": "uuid",
    "activity_id": "uuid",
    "user_id": "uuid",
    "status": "registered",
    "registered_at": "2025-01-05T10:30:00Z"
  },
  "message": "Successfully registered for activity"
}
```

### QR Code Scan
```http
POST /api/activities/{activity_id}/scan
Cookie: session_id=admin-session-id
```

**Request Body:**
```json
{
  "qr_secret": "user-qr-secret-from-scan",
  "scan_type": "check_in"
}
```

**Response:**
```json
{
  "success": true,
  "user": {
    "id": "uuid",
    "name": "John Doe",
    "student_id": "64123456789"
  },
  "participation": {
    "status": "checked_in",
    "checked_in_at": "2025-01-10T09:15:00Z"
  },
  "message": "Check-in successful"
}
```

---

## Real-time Events (SSE)

### Connect to SSE
```http
GET /api/sse/events
Cookie: session_id=your-session-id
```

**Headers:**
```
Accept: text/event-stream
Cache-Control: no-cache
```

### Admin SSE
```http
GET /api/sse/admin
Cookie: session_id=admin-session-id
```

### SSE Event Types

#### Notification
```
event: notification
data: {
  "event_type": "notification",
  "data": {
    "title": "New Activity Available",
    "message": "Programming Workshop is now open for registration",
    "notification_type": "info",
    "action_url": "/activities/uuid",
    "expires_at": "2025-01-10T00:00:00Z"
  },
  "timestamp": "2025-01-05T10:30:00Z"
}
```

#### Session Update
```
event: session_update
data: {
  "event_type": "session_update", 
  "data": {
    "session_id": "your-session-id",
    "action": "force_logout",
    "reason": "Security policy violation",
    "new_expires_at": null
  },
  "timestamp": "2025-01-05T10:30:00Z"
}
```

#### Activity Update
```
event: activity_update
data: {
  "event_type": "activity_update",
  "data": {
    "activity_id": "uuid",
    "title": "Programming Workshop", 
    "update_type": "started",
    "message": "Activity has started. Check-in is now available."
  },
  "timestamp": "2025-01-10T09:00:00Z"
}
```

---

## Faculties & Departments

### Get Faculties
```http
GET /api/faculties
```

**Response:**
```json
{
  "faculties": [
    {
      "id": "uuid",
      "name": "Computer Science",
      "code": "CS",
      "description": "Computer Science and Technology",
      "departments": [
        {
          "id": "uuid",
          "name": "Software Engineering",
          "code": "SE"
        }
      ],
      "created_at": "2025-01-01T00:00:00Z"
    }
  ]
}
```

### Create Faculty (Super Admin)
```http
POST /api/faculties
Cookie: session_id=admin-session-id
```

**Request Body:**
```json
{
  "name": "Computer Science",
  "code": "CS", 
  "description": "Computer Science and Technology"
}
```

---

## Users Management (Admin)

### Get Users
```http
GET /api/admin/users?limit=50&role=student&faculty_id=uuid
Cookie: session_id=admin-session-id
```

**Query Parameters:**
- `limit`: Number of users (default: 50)
- `offset`: Pagination offset (default: 0)
- `role`: Filter by role (student, admin)
- `faculty_id`: Filter by faculty
- `search`: Search by name or email

**Response:**
```json
{
  "users": [
    {
      "id": "uuid",
      "student_id": "64123456789",
      "email": "student@university.ac.th",
      "first_name": "John",
      "last_name": "Doe",
      "department": {
        "id": "uuid",
        "name": "Software Engineering"
      },
      "admin_role": null,
      "created_at": "2025-01-05T10:00:00Z",
      "last_login": "2025-01-05T10:30:00Z"
    }
  ],
  "total_count": 150
}
```

---

## Error Responses

### 400 Bad Request
```json
{
  "error": "ValidationError",
  "message": "Invalid input data",
  "details": {
    "student_id": "Student ID must be 11 digits",
    "email": "Invalid email format"
  }
}
```

### 401 Unauthorized
```json
{
  "error": "Unauthorized",
  "message": "Authentication required"
}
```

### 403 Forbidden
```json
{
  "error": "Forbidden", 
  "message": "Insufficient permissions",
  "required_permission": "ManageActivities"
}
```

### 404 Not Found
```json
{
  "error": "NotFound",
  "message": "Resource not found"
}
```

### 429 Too Many Requests
```json
{
  "error": "RateLimitExceeded",
  "message": "Too many requests, please try again later",
  "retry_after": 60
}
```

### 500 Internal Server Error
```json
{
  "error": "InternalServerError", 
  "message": "An unexpected error occurred",
  "request_id": "uuid"
}
```

---

## Rate Limiting

The API implements rate limiting based on:
- **Authentication endpoints**: 5 requests per minute
- **General API**: 100 requests per minute per session
- **Admin endpoints**: 200 requests per minute per session

Rate limit headers are included in responses:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1641024000
```

---

## Permissions System

### Admin Levels
- **SuperAdmin**: Full system access
- **FacultyAdmin**: Faculty-specific management
- **RegularAdmin**: Basic admin functions

### Available Permissions
- `ViewSystemReports` - View system-wide reports
- `ManageAllFaculties` - Manage all faculties
- `ManageFacultyUsers` - Manage faculty users
- `ManageFacultyActivities` - Manage faculty activities
- `ManageUsers` - Manage all users
- `ManageActivities` - Manage all activities
- `ManageAdmins` - Manage admin accounts
- `ManageSessions` - View and manage sessions
- `ViewAllReports` - View all reports
- `ScanQrCodes` - Scan QR codes for attendance
- `ViewAssignedActivities` - View assigned activities
- `ViewProfile` - View user profile
- `UpdateProfile` - Update user profile

---

## WebSocket Alternative

While the main real-time communication uses SSE, WebSocket endpoints are also available:

```
ws://localhost:3000/ws/events?session_id=your-session-id
ws://localhost:3000/ws/admin?session_id=admin-session-id
```

WebSocket messages follow the same format as SSE events but in JSON format.

---

## SDK Examples

### JavaScript/TypeScript
```typescript
// Authentication
const response = await fetch('/api/auth/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  credentials: 'include',
  body: JSON.stringify({
    student_id: '64123456789',
    password: 'password'
  })
});

// SSE Connection
const eventSource = new EventSource('/api/sse/events', {
  withCredentials: true
});

eventSource.addEventListener('notification', (event) => {
  const data = JSON.parse(event.data);
  console.log('Notification:', data);
});
```

### cURL Examples
```bash
# Login
curl -c cookies.txt -X POST \
  -H "Content-Type: application/json" \
  -d '{"student_id":"64123456789","password":"password"}' \
  http://localhost:3000/api/auth/login

# Get activities
curl -b cookies.txt \
  http://localhost:3000/api/activities

# SSE Connection
curl -N -b cookies.txt \
  http://localhost:3000/api/sse/events
```

---

For more information, see the [Development Setup Guide](DEVELOPMENT_SETUP.md).