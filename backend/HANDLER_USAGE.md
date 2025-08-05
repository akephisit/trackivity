# Backend Handlers Usage Guide

ฉันได้สร้าง handlers ครบถ้วนสำหรับ backend ตาม routes ที่กำหนด นี่คือการใช้งานของแต่ละ handler:

## 1. Admin Handlers (`admin.rs`)

### Dashboard Statistics
```
GET /api/admin/dashboard
```
- ดึงสถิติแดชบอร์ดรวม (total users, activities, sessions)
- แสดงกิจกรรมล่าสุดและยอดนิยม
- ต้องใช้สิทธิ์ AdminUser

### Admin Users Management
```
GET /api/admin/users?limit=50&offset=0&search=john
```
- ดึงรายการ users พร้อมข้อมูล admin roles
- รองรับการค้นหาและ pagination
- แสดงสถานะ active/inactive และข้อมูลล่าสุด

### Admin Activities Overview
```
GET /api/admin/activities?status=ongoing&search=workshop
```
- ดึงกิจกรรมทั้งหมดพร้อมจำนวนผู้เข้าร่วม
- สามารถกรองตาม status และค้นหา
- แสดงข้อมูลผู้สร้างและหน่วยงาน

### Admin Sessions Information
```
GET /api/admin/sessions?limit=100&search=user@example.com
```
- ดึงข้อมูล sessions ทั้งหมด (สำหรับ SuperAdmin)
- แสดงข้อมูล device, IP, และสถานะ
- รองรับการค้นหาและกรอง

## 2. User Handlers (`user.rs`)

### User Management
```
GET /api/users?limit=50&department_id=uuid&search=john
POST /api/users
PUT /api/users/:id
DELETE /api/users/:id
```
- CRUD operations สำหรับ users
- ค้นหาตาม department และข้อมูลส่วนตัว
- แสดงจำนวนกิจกรรมที่เข้าร่วม

### User Details
```
GET /api/users/:id
```
- ดึงข้อมูลผู้ใช้แบบละเอียด
- รวม admin role, department, faculty
- แสดงสถิติการเข้าร่วมกิจกรรม

### QR Code Generation
```
GET /api/users/:id/qr
```
- สร้าง QR code สำหรับ check-in/out
- รองรับทั้ง admin และ user เจ้าของ
- ส่งกลับเป็น base64 image

## 3. Activity Handlers (`activity.rs`)

### Activity Management
```
GET /api/activities?status=published&faculty_id=uuid
POST /api/activities
PUT /api/activities/:id
DELETE /api/activities/:id
```
- CRUD operations สำหรับกิจกรรม
- กรองตาม status, faculty, department
- ตรวจสอบสิทธิ์การแก้ไข (creator หรือ admin)

### Activity Details
```
GET /api/activities/:id
```
- ดึงข้อมูลกิจกรรมพร้อมสถานะการลงทะเบียนของ user
- แสดงจำนวนผู้เข้าร่วมปัจจุบัน
- ข้อมูलหน่วยงานและผู้สร้าง

### Participation Management
```
GET /api/activities/:id/participations
POST /api/activities/:id/participate
POST /api/activities/:id/scan
```
- ดูรายการผู้เข้าร่วม (สำหรับ creator/admin)
- ลงทะเบียนเข้าร่วมกิจกรรม
- สแกน QR code สำหรับ check-in/out

## 4. Admin Session Handlers (`admin_session.rs`)

### Session Management
```
GET /api/admin/sessions
DELETE /api/admin/sessions/:id
POST /api/admin/sessions/cleanup
```
- ดูรายการ sessions ทั้งหมด
- ยกเลิก session เฉพาะ
- ทำความสะอาด expired sessions

## Response Format

ทุก handler ใช้ response format เดียวกัน:

```json
{
  "status": "success|error",
  "data": { ... },
  "message": "Operation completed successfully"
}
```

## Authentication & Authorization

- **AdminUser**: สิทธิ์ admin ขั้นพื้นฐาน
- **SuperAdminUser**: สิทธิ์ admin สูงสุด  
- **SessionUser**: ผู้ใช้งานทั่วไป (มี session)

## Features ที่รองรับ

1. **Pagination**: `limit` และ `offset` parameters
2. **Search**: ค้นหาในฟิลด์ที่เกี่ยวข้อง
3. **Filtering**: กรองตาม status, faculty, department
4. **Error Handling**: จัดการ errors แบบครอบคลุม
5. **Permission Checks**: ตรวจสอบสิทธิ์การเข้าถึง
6. **QR Code Support**: สร้างและตรวจสอบ QR codes
7. **Activity Tracking**: ติดตามการเข้าร่วมกิจกรรม

## Database Integration

ใช้ `sqlx` สำหรับ database operations:
- PostgreSQL เป็น primary database
- Redis สำหรับ session management
- Type-safe queries และ proper error handling
- Support สำหรับ enum types และ JSON fields

ทุก handler ได้รับการออกแบบให้รองรับการใช้งานจริงพร้อม production-ready features.