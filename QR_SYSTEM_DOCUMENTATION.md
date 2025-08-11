# QR Code System Documentation

## ภาพรวมของระบบ

ระบบ QR Code ของ Trackivity เป็นระบบที่ให้ผู้ใช้สามารถสร้าง QR Code ด้วยตนเองบนฝั่ง client และใช้สำหรับการ check-in เข้าร่วมกิจกรรมต่าง ๆ ระบบนี้ใช้หลักการ client-side generation เพื่อลดภาระการทำงานของเซิร์ฟเวอร์และเพิ่มประสิทธิภาพในการใช้งาน

## คุณสมบัติหลัก

### 1. Client-Side QR Generation
- **เก็บเฉพาะ unique identifier**: เซิร์ฟเวอร์เก็บเฉพาะ secret key และ user_id ไม่เก็บภาพ QR Code
- **Frontend/Mobile geneartion**: แอปพลิเคชันฝั่ง client สร้าง QR Code จากข้อมูลที่ได้รับ
- **รูปแบบข้อมูล QR**: `{"user_id": "xxx", "student_id": "xxx", "secret": "xxx", "timestamp": "xxx"}`
- **การตรวจสอบฝั่งเซิร์ฟเวอร์**: เซิร์ฟเวอร์ตรวจสอบความถูกต้องของ secret และ timestamp

### 2. Activity Management REST Endpoints
- **GET /api/qr/generate**: สร้างข้อมูล QR สำหรับผู้ใช้
- **POST /api/qr/refresh**: รีเฟรช secret key ของผู้ใช้
- **POST /api/activities/{id}/checkin**: endpoint สำหรับการ check-in ด้วย QR Code
- **GET /api/admin/activities/assigned**: ดูกิจกรรมที่ admin ได้รับมอบหมาย

### 3. Activity Assignment System
- **Regular admin**: สามารถสแกน QR Code เฉพาะกิจกรรมที่สร้างเองหรือในขอบเขตคณะที่มีสิทธิ์
- **Faculty admin**: สามารถจัดการกิจกรรมในขอบเขตคณะของตนเอง
- **Super admin**: สามารถเข้าถึงกิจกรรมทั้งหมดในระบบ
- **Permission-based access**: ใช้ระบบสิทธิ์แบบ type-safe ใน Rust

### 4. QR Scanning Flow
```
1. Admin เปิดแอป scanner
2. สแกน QR Code ของนักศึกษา
3. POST ไปยัง /api/activities/{id}/checkin พร้อมข้อมูล QR
4. เซิร์ฟเวอร์ตรวจสอบ signature และดูว่านักศึกษามีอยู่หรือไม่
5. บันทึกการเข้าร่วมในฐานข้อมูลด้วย sqlx
6. ส่ง SSE notification ไปยัง connections ที่เกี่ยวข้อง
```

## Rust QR Validation System

### โครงสร้างข้อมูล

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientQrData {
    pub user_id: uuid::Uuid,
    pub student_id: String,
    pub secret: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrGenerationResponse {
    pub qr_data: String,
    pub expires_at: u64,
}
```

### การตรวจสอบความปลอดภัย

1. **Timestamp Validation**: ป้องกัน replay attacks โดยตรวจสอบว่า QR Code ไม่หมดอายุ
2. **Secret Verification**: ตรวจสอบ secret key กับที่เก็บในฐานข้อมูล
3. **User Existence Check**: ยืนยันว่าผู้ใช้มีอยู่จริงในระบบ
4. **Faculty/Activity Permission**: ตรวจสอบสิทธิ์การเข้าถึงกิจกรรม
5. **Memory-safe Processing**: ใช้ Rust สำหรับการจัดการ string ที่ปลอดภัย

### ฟีเจอร์ประสิทธิภาพ

- **Zero-copy JSON parsing**: ใช้ serde สำหรับการประมวลผล JSON ที่รวดเร็ว
- **Async/await**: การประมวลผลแบบ non-blocking
- **Concurrent request processing**: รองรับการประมวลผล request หลายตัวพร้อมกัน
- **Error handling with Result<T, E>**: การจัดการข้อผิดพลาดที่เป็นระบบ

## การใช้งาน API

### 1. สร้าง QR Code สำหรับผู้ใช้

```bash
GET /api/qr/generate
Authorization: Bearer <session_token>
```

Response:
```json
{
  "status": "success",
  "data": {
    "qr_data": "{\"user_id\":\"...\",\"student_id\":\"...\",\"secret\":\"...\",\"timestamp\":...}",
    "expires_at": 1642694400,
    "user_info": {
      "student_id": "2567123456",
      "name": "สมชาย ใจดี"
    }
  }
}
```

### 2. รีเฟรช Secret Key

```bash
POST /api/qr/refresh
Authorization: Bearer <session_token>
```

Response:
```json
{
  "status": "success",
  "message": "QR secret refreshed successfully. Previous QR codes are now invalid."
}
```

### 3. Check-in ด้วย QR Code

```bash
POST /api/activities/{activity_id}/checkin
Authorization: Bearer <admin_session_token>
Content-Type: application/json

{
  "qr_data": "{\"user_id\":\"...\",\"student_id\":\"...\",\"secret\":\"...\",\"timestamp\":...}"
}
```

Response:
```json
{
  "status": "success",
  "data": {
    "success": true,
    "message": "Successfully checked in",
    "user_name": "สมชาย ใจดี",
    "student_id": "2567123456",
    "participation_status": "CheckedIn",
    "checked_in_at": "2024-01-20T10:30:00Z"
  }
}
```

### 4. ดูกิจกรรมที่ได้รับมอบหมาย

```bash
GET /api/admin/activities/assigned
Authorization: Bearer <admin_session_token>
```

Response:
```json
{
  "status": "success",
  "data": {
    "activities": [
      {
        "id": "activity-uuid",
        "title": "งานปฐมนิเทศนักศึกษาใหม่",
        "description": "กิจกรรมต้อนรับนักศึกษาใหม่ประจำปี",
        "location": "หอประชุมใหญ่",
        "start_time": "2024-01-25T08:00:00Z",
        "end_time": "2024-01-25T16:00:00Z",
        "status": "published",
        "max_participants": 500,
        "current_participants": 234,
        "faculty_name": "คณะวิศวกรรมศาสตร์",
        "department_name": "วิศวกรรมคอมพิวเตอร์"
      }
    ],
    "total_count": 1,
    "admin_type": "regular_admin"
  }
}
```

## Real-time Notifications (SSE)

### การแจ้งเตือนเมื่อมี Check-in

เมื่อมีการ check-in สำเร็จ ระบบจะส่ง SSE notification ไปยัง:

1. **Admins ที่มีสิทธิ์ ManageActivities**
2. **Admins ที่มีสิทธิ์ ViewParticipations**
3. **Admins ที่อยู่ในคณะเดียวกันกับกิจกรรม**

Format ของ notification:
```json
{
  "event_type": "activity_checkin",
  "data": {
    "activity_id": "activity-uuid",
    "activity_title": "งานปฐมนิเทศนักศึกษาใหม่",
    "user_name": "สมชาย ใจดี",
    "student_id": "2567123456",
    "action": "checked_in",
    "checked_in_at": "2024-01-20T10:30:00Z"
  },
  "timestamp": "2024-01-20T10:30:00Z"
}
```

## ระบบความปลอดภัย

### 1. QR Code Security
- **Timestamp-based Expiry**: QR Code หมดอายุภายใน 5 นาที
- **Secret Verification**: ตรวจสอบ secret key กับฐานข้อมูล
- **Unique User Binding**: QR Code ผูกกับ user_id เฉพาะ
- **No Image Storage**: ไม่เก็บภาพ QR Code บนเซิร์ฟเวอร์

### 2. API Security
- **Session-based Authentication**: ใช้ Redis session store
- **Permission-based Authorization**: ตรวจสอบสิทธิ์ทุก endpoint
- **Faculty Scope Validation**: จำกัดการเข้าถึงตามขอบเขตคณะ
- **Input Validation**: ตรวจสอบ input ทุกประเภท

### 3. Database Security
- **Prepared Statements**: ใช้ sqlx เพื่อป้องกัน SQL injection
- **Type-safe Queries**: ใช้ Rust type system เพื่อความปลอดภัย
- **Connection Pooling**: จัดการ database connections อย่างมีประสิทธิภาพ

## การติดตั้งและการใช้งาน

### 1. Database Schema
ระบบต้องการตาราง users ที่มี field `qr_secret`:

```sql
ALTER TABLE users ADD COLUMN IF NOT EXISTS qr_secret VARCHAR(255) NOT NULL DEFAULT '';
```

### 2. Environment Variables
```bash
DATABASE_URL=postgresql://user:password@localhost/trackivity
REDIS_URL=redis://localhost:6379
SESSION_TIMEOUT=3600
```

### 3. การเริ่มต้นระบบ
```bash
cd backend
cargo build --release
./target/release/trackivity
```

## การพัฒนาและการทดสอบ

### 1. การทดสอบ QR Code Functions
```bash
cargo test qr_code_tests
```

### 2. การทดสอบ API Endpoints
```bash
# ทดสอบการสร้าง QR
curl -X GET http://localhost:8080/api/qr/generate \
  -H "Authorization: Bearer <session_token>"

# ทดสอบการ check-in
curl -X POST http://localhost:8080/api/activities/{id}/checkin \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{"qr_data": "..."}'
```

### 3. การ Debug SSE
เชื่อมต่อ SSE endpoint เพื่อดูการแจ้งเตือนแบบ real-time:
```javascript
const eventSource = new EventSource('/api/sse/session_id');
eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('SSE Event:', data);
};
```

## ข้อมูลเพิ่มเติม

### Repository Structure
```
/backend/src/
├── handlers/
│   ├── qr_activity.rs    # QR Code handlers
│   ├── activity.rs       # Activity management
│   └── sse.rs           # Server-Sent Events
├── utils/
│   └── qr.rs            # QR Code utilities
├── models/
│   ├── user.rs          # User model with qr_secret
│   └── participation.rs # Participation tracking
└── routes/
    └── mod.rs           # API routing
```

### Dependencies
```toml
[dependencies]
# Core web framework
axum = { version = "0.8", features = ["macros", "tokio", "tower-log"] }

# Database & Redis
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "migrate"] }
redis = { version = "0.26", features = ["tokio-comp", "connection-manager"] }

# Serialization & UUID
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }

# Security & QR Code
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"
qrcode = "0.14"

# Async runtime
tokio = { version = "1.47", features = ["full"] }
```

## สรุป

ระบบ QR Code ของ Trackivity ถูกออกแบบมาให้มีประสิทธิภาพสูงและปลอดภัย โดยใช้หลักการ:

1. **Client-side Generation**: ลดภาระเซิร์ฟเวอร์และเพิ่มความเร็ว
2. **Security First**: การตรวจสอบหลายชั้นด้วย timestamp และ secret
3. **Real-time Updates**: SSE สำหรับการแจ้งเตือนแบบทันที
4. **Permission-based Access**: ระบบสิทธิ์ที่ยืดหยุ่นและปลอดภัย
5. **Type-safe Implementation**: ใช้ Rust เพื่อความปลอดภัยและประสิทธิภาพ

ระบบนี้พร้อมใช้งานและสามารถรองรับการใช้งานจริงในสถานการณ์ต่าง ๆ ได้อย่างมีประสิทธิภาพ