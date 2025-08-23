# Thai Prefix Implementation Test Plan

## Overview
This document outlines the comprehensive test plan for the Thai prefix (คำนำหน้า) implementation across the backend system.

## Changes Made

### 1. Database Migration
- **File**: `migrations/002_add_user_prefix.sql`
- **Changes**:
  - Created `user_prefix` ENUM type with Thai prefixes
  - Added `prefix` column to `users` table with default 'นาย'
  - Added index for better query performance
  - Updated existing users with appropriate defaults

### 2. User Model Updates
- **File**: `src/models/user.rs`
- **Changes**:
  - Added `UserPrefix` enum with Thai values
  - Updated `User` struct to include `prefix` field
  - Updated `CreateUser`, `UpdateUser`, and `UserResponse` structs
  - Added `full_name_with_prefix` computed field
  - Added helper method `to_thai_string()` for prefix display

### 3. User Handler Updates
- **File**: `src/handlers/user.rs`
- **Changes**:
  - Updated all CRUD operations to handle prefix field
  - Modified SQL queries to include prefix column
  - Updated `UserWithDetails` struct for admin views
  - Added prefix handling in create, update, and get operations

### 4. Activity Handler Updates
- **File**: `src/handlers/activity.rs`
- **Changes**:
  - Updated `ParticipationWithUser` struct to include prefix data
  - Modified participation queries to include user prefix
  - Updated QR scan responses to include prefixed names
  - Added `user_name_with_prefix` field for display

### 5. Participation Model Updates
- **File**: `src/models/participation.rs`
- **Changes**:
  - Added `ParticipationWithUserDetails` struct
  - Included comprehensive user prefix information
  - Added full name with prefix fields

## API Endpoints to Test

### User Management Endpoints

#### 1. Get All Users
```bash
GET /api/users
```
**Expected**: Should return users with prefix, full_name_with_prefix fields

#### 2. Get User by ID
```bash
GET /api/users/{id}
```
**Expected**: Should return user with prefix information

#### 3. Create User
```bash
POST /api/users
```
**Body**:
```json
{
  "student_id": "64070001",
  "email": "test@example.com",
  "password": "testpassword",
  "prefix": "Mr",
  "first_name": "สมชาย",
  "last_name": "ใจดี",
  "department_id": null
}
```
**Expected**: Should create user with specified prefix

#### 4. Update User
```bash
PUT /api/users/{id}
```
**Body**:
```json
{
  "prefix": "Dr",
  "first_name": "สมชาย"
}
```
**Expected**: Should update user prefix and recalculate full name

### Activity Endpoints

#### 1. Get Activity Participations
```bash
GET /api/activities/{id}/participations
```
**Expected**: Should return participants with prefix information and prefixed names

#### 2. QR Code Scan
```bash
POST /api/activities/{id}/scan
```
**Expected**: Should return user information with prefix in response

## Database Schema Verification

### 1. Verify Enum Type
```sql
SELECT enumlabel FROM pg_enum WHERE enumtypid = 'user_prefix'::regtype;
```
**Expected**: Should show all Thai prefixes

### 2. Verify Column Addition
```sql
\d users
```
**Expected**: Should show `prefix` column with `user_prefix` type

### 3. Test Default Values
```sql
SELECT student_id, prefix, first_name, last_name FROM users LIMIT 5;
```
**Expected**: Should show users with appropriate prefix values

## Frontend Integration Points

### 1. Activity List Page (รายการกิจกรรม)
- User names should display with Thai prefixes
- Participation lists should show prefixed names
- Student activity pages should use proper titles

### 2. Student Activity Pages
- Student names should display with appropriate prefixes
- Profile information should show prefix selection
- Registration forms should include prefix options

## Test Data Examples

### Thai Prefixes Mapping
- `Mr` → "นาย"
- `Mrs` → "นาง"  
- `Miss` → "นางสาว"
- `Dr` → "ดร."
- `Professor` → "ศาสตราจารย์"
- `AssociateProfessor` → "รองศาสตราจารย์"
- `AssistantProfessor` → "ผู้ช่วยศาสตราจารย์"
- `Lecturer` → "อาจารย์"
- `Generic` → "คุณ"

### Sample Full Names with Prefixes
- "นายสมชาย ใจดี"
- "นางสาวสุดา ใจงาม"
- "ดร.วิชัย นักวิชา"
- "อาจารย์ปราณี ครูใหญ่"

## Error Handling Tests

### 1. Invalid Prefix Value
```bash
POST /api/users
```
**Body**:
```json
{
  "prefix": "InvalidPrefix",
  ...
}
```
**Expected**: Should return validation error

### 2. Missing Required Fields
**Expected**: Should return appropriate error messages

## Performance Considerations

### 1. Query Performance
- Test that prefix queries use the new index
- Verify that JOIN operations with prefix data are efficient

### 2. Database Load
- Test with multiple users having different prefixes
- Verify that prefix enum operations are performant

## Migration Testing

### 1. Fresh Database
- Apply migration to empty database
- Verify schema creation

### 2. Existing Data
- Test migration with existing user data
- Verify that existing users get appropriate default prefixes

## Success Criteria

1. ✅ All API endpoints return prefix information correctly
2. ✅ Database migration applies successfully
3. ✅ User CRUD operations handle prefixes properly
4. ✅ Activity participation data includes user prefixes
5. ✅ QR code scanning returns prefixed user names
6. ✅ All existing functionality remains intact
7. ✅ Frontend can display Thai prefixes correctly

## Notes

- All Thai text should be properly encoded in UTF-8
- Prefix enum values are stored in Thai characters in database
- API responses include both enum values and Thai strings
- Full name concatenation includes proper spacing
- Backward compatibility is maintained for existing data