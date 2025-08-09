# Faculty Scope Authorization Enhancement

## Overview

This document describes the enhanced faculty scope authorization system implemented in the session middleware. The system provides comprehensive faculty-scoped access control for admin operations, ensuring that faculty administrators can only access resources within their assigned faculty scope.

## Key Features

### 1. **FacultyScopedAdminUser Extractor**
- Automatically extracts and validates faculty access from path parameters
- Combines authentication, admin role validation, and faculty scope checking
- Provides compile-time safety and runtime efficiency

### 2. **Helper Functions for Manual Validation**
- `has_faculty_access()` - Check if user has access to a specific faculty
- `validate_faculty_access()` - Validate access and return appropriate error
- `get_accessible_faculty_ids()` - Get list of accessible faculty IDs

### 3. **Enhanced Middleware Support**
- Path-based faculty validation middleware
- Automatic faculty_id extraction from URL patterns
- Integration with existing authentication system

## Authorization Rules

### SuperAdmin
- ✅ Access to **all faculties**
- ✅ Can perform any operation on any faculty
- ✅ No restrictions on faculty scope

### FacultyAdmin
- ✅ Access to **assigned faculty only**
- ❌ Cannot access other faculties
- ✅ Full admin privileges within their faculty

### RegularAdmin
- ✅ Access to **assigned faculty only** (same as FacultyAdmin)
- ❌ Cannot access other faculties
- ✅ Admin privileges within their faculty (based on permissions)

### Regular Users
- ❌ No faculty admin access
- ❌ Cannot perform faculty admin operations

## Implementation Patterns

### Pattern 1: Using FacultyScopedAdminUser Extractor

```rust
use crate::middleware::session::FacultyScopedAdminUser;

pub async fn get_faculty_departments(
    faculty_scoped_admin: FacultyScopedAdminUser, // Automatic validation
    Query(pagination): Query<PaginationQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let faculty_id = faculty_scoped_admin.faculty_id; // Guaranteed accessible
    
    // No additional authorization needed - extractor handles it
    let departments = query_departments_by_faculty(
        &app_state.db_pool, 
        faculty_id, 
        pagination.page.unwrap_or(1), 
        pagination.per_page.unwrap_or(10)
    ).await?;
    
    Ok(Json(json!({
        "status": "success",
        "data": { "departments": departments }
    })))
}
```

**Advantages:**
- Automatic validation
- Clean, declarative code
- Compile-time safety
- Less boilerplate

**When to use:**
- Standard CRUD operations on faculty resources
- When faculty_id is in path parameters
- Simple authorization requirements

### Pattern 2: Manual Validation with Helper Functions

```rust
use crate::middleware::session::{AdminUser, validate_faculty_access};

pub async fn update_faculty_settings(
    Path(faculty_id): Path<Uuid>,
    admin_user: AdminUser,
    State(app_state): State<AppState>,
    Json(request): Json<UpdateRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Manual validation with custom error handling
    validate_faculty_access(&admin_user.session_user, faculty_id)?;
    
    // Additional business logic validation
    if request.requires_special_permission() {
        check_special_permission(&admin_user)?;
    }
    
    // Perform operation
    update_faculty(&app_state.db_pool, faculty_id, &request).await?;
    
    Ok(Json(json!({"status": "success"})))
}
```

**Advantages:**
- More control over validation logic
- Custom error handling
- Complex authorization scenarios
- Additional validation steps

**When to use:**
- Complex business logic requirements
- Multiple validation steps needed
- Custom error messages required
- Non-standard path parameter patterns

### Pattern 3: Conditional Access Based on Faculty Scope

```rust
use crate::middleware::session::{SessionUser, has_faculty_access, get_accessible_faculty_ids};

pub async fn bulk_faculty_operation(
    admin_user: AdminUser,
    Json(faculty_ids): Json<Vec<Uuid>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let accessible_faculties = get_accessible_faculty_ids(&admin_user.session_user);
    
    let allowed_faculties = match accessible_faculties {
        None => faculty_ids, // SuperAdmin - all allowed
        Some(allowed) => {
            // Filter to only accessible faculties
            faculty_ids.into_iter()
                .filter(|id| allowed.contains(id))
                .collect()
        }
    };
    
    if allowed_faculties.is_empty() {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Process only allowed faculties
    process_faculties(&allowed_faculties).await?;
    
    Ok(Json(json!({
        "processed_count": allowed_faculties.len()
    })))
}
```

**Advantages:**
- Handles multi-faculty operations
- Graceful filtering of accessible resources
- Flexible for different admin levels

**When to use:**
- Bulk operations across multiple faculties
- Operations that might span different scopes
- Listing/filtering scenarios

### Pattern 4: Middleware-Level Faculty Validation

```rust
use crate::middleware::session::require_faculty_access_from_path;

fn create_router() -> Router {
    Router::new()
        .route("/api/admin/faculties/:faculty_id/protected", get(handler))
        .layer(axum::middleware::from_fn(require_faculty_access_from_path()))
}

pub async fn handler(
    Path(faculty_id): Path<Uuid>,
    session_user: SessionUser,
) -> Json<serde_json::Value> {
    // If we reach here, middleware already validated access
    Json(json!({
        "message": "Access granted",
        "faculty_id": faculty_id
    }))
}
```

**Advantages:**
- Centralized validation
- Applied to multiple routes
- Early rejection of unauthorized requests

**When to use:**
- Protecting entire route groups
- Consistent validation across endpoints
- When you want to fail fast

## Path Parameter Patterns

The system automatically extracts `faculty_id` from these URL patterns:

```
/api/admin/faculties/{faculty_id}/departments
/api/admin/faculties/{faculty_id}/statistics  
/api/faculties/{faculty_id}/settings
/faculties/{faculty_id}/info
```

### Custom Path Parameter Extraction

For non-standard patterns, you can extract manually:

```rust
pub async fn custom_handler(
    Path(params): Path<HashMap<String, String>>,
    admin_user: AdminUser,
) -> Result<Json<Value>, StatusCode> {
    let faculty_id = params.get("custom_faculty_param")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    validate_faculty_access(&admin_user.session_user, faculty_id)?;
    
    // Continue with handler logic
}
```

## Error Responses

### Unauthorized (401)
- User not authenticated
- Invalid session

### Forbidden (403)
- User authenticated but lacks admin role
- Admin lacks access to specified faculty
- FacultyAdmin trying to access different faculty

### Bad Request (400)
- Invalid faculty_id format in path
- Missing required path parameters

## Testing

The system includes comprehensive tests:

```bash
# Run faculty scope tests
cargo test test_faculty_scope

# Run specific test
cargo test test_super_admin_faculty_access
cargo test test_faculty_admin_different_faculty_access
```

## Integration Examples

### Route Definition

```rust
use crate::middleware::session::FacultyScopedAdminUser;

Router::new()
    // Automatic validation
    .route("/api/admin/faculties/:faculty_id/departments", 
           get(get_departments).post(create_department))
    
    // Manual validation  
    .route("/api/admin/faculties/:faculty_id/stats",
           get(get_stats_with_manual_validation))
    
    // Middleware validation
    .route("/api/admin/faculties/:faculty_id/protected",
           get(protected_handler))
    .layer(axum::middleware::from_fn(require_faculty_access_from_path()))
```

### Database Integration

```rust
pub async fn create_department(
    faculty_scoped_admin: FacultyScopedAdminUser,
    State(app_state): State<AppState>,
    Json(request): Json<CreateDepartmentRequest>,
) -> Result<Json<Value>, StatusCode> {
    let faculty_id = faculty_scoped_admin.faculty_id;
    
    // Create department with validated faculty_id
    let department = sqlx::query_as::<_, Department>(
        "INSERT INTO departments (faculty_id, name, code) 
         VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(faculty_id)
    .bind(&request.name)
    .bind(&request.code)
    .fetch_one(&app_state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json!({
        "status": "success", 
        "data": department
    })))
}
```

## Performance Considerations

### Extractor Performance
- `FacultyScopedAdminUser`: Single path parameter extraction + validation
- Minimal overhead compared to manual validation
- Path parsing cached by Axum

### Database Impact
- No additional database queries for faculty validation
- Uses existing session data and admin roles
- Faculty_id already available in SessionUser

### Memory Usage
- Minimal additional memory overhead
- Reuses existing authentication structures
- No persistent state required

## Security Considerations

### Path Parameter Validation
- UUID format validation prevents injection
- Invalid UUIDs return 400 Bad Request
- No SQL injection risk

### Authorization Bypass Prevention
- Multiple validation layers
- Extractor + helper functions + middleware options
- Consistent validation logic across all patterns

### Session Security
- Integrates with existing session management
- No separate authentication mechanism
- Leverages proven session validation

## Migration Guide

### From Basic Admin Authorization

**Before:**
```rust
pub async fn get_departments(
    admin_user: AdminUser,
    Path(faculty_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    // Manual faculty validation
    if admin_user.admin_role.faculty_id != Some(faculty_id) 
       && !matches!(admin_user.admin_role.admin_level, AdminLevel::SuperAdmin) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Handler logic
}
```

**After:**
```rust
pub async fn get_departments(
    faculty_scoped_admin: FacultyScopedAdminUser,
) -> Result<Json<Value>, StatusCode> {
    let faculty_id = faculty_scoped_admin.faculty_id;
    // Handler logic - no manual validation needed
}
```

### Adding Faculty Scope to Existing Handlers

1. **Replace AdminUser with FacultyScopedAdminUser** for automatic validation
2. **Use helper functions** for complex scenarios  
3. **Add middleware** for route-level protection
4. **Update tests** to include faculty scope scenarios

## Best Practices

### 1. Choose the Right Pattern
- Use `FacultyScopedAdminUser` for simple cases
- Use helper functions for complex validation
- Use middleware for route group protection

### 2. Consistent Error Handling
```rust
// Good: Consistent error responses
validate_faculty_access(&user, faculty_id)
    .map_err(|_| (StatusCode::FORBIDDEN, "Access denied to faculty"))?;

// Avoid: Inconsistent error types
if !has_faculty_access(&user, faculty_id) {
    return Err(StatusCode::UNAUTHORIZED); // Wrong status code
}
```

### 3. Database Query Optimization
```rust
// Good: Use validated faculty_id in queries
let departments = sqlx::query_as!(
    Department,
    "SELECT * FROM departments WHERE faculty_id = $1",
    faculty_scoped_admin.faculty_id  // Already validated
).fetch_all(&pool).await?;

// Avoid: Re-validating in database queries
let departments = sqlx::query_as!(
    Department, 
    "SELECT d.* FROM departments d 
     JOIN admin_roles ar ON (ar.faculty_id = d.faculty_id OR ar.admin_level = 'SuperAdmin')
     WHERE d.faculty_id = $1 AND ar.user_id = $2",
    faculty_id, user_id  // Redundant validation
).fetch_all(&pool).await?;
```

### 4. Testing
- Test all admin levels (SuperAdmin, FacultyAdmin, RegularAdmin)  
- Test cross-faculty access attempts
- Test invalid faculty_id formats
- Test edge cases (admin without assigned faculty)

## Troubleshooting

### Common Issues

**"Bad Request" on valid requests**
- Check path parameter name matches `:faculty_id`
- Verify UUID format in URL
- Ensure route pattern is correct

**"Forbidden" for SuperAdmin**  
- Check admin_role is properly loaded in session
- Verify AdminLevel enum values
- Check database admin_roles table

**"Unauthorized" instead of "Forbidden"**
- Check session middleware is applied  
- Verify extractor order in handler function
- Check authentication headers/cookies

### Debug Tips

```rust
// Add logging for debugging
pub async fn debug_handler(
    admin_user: AdminUser,
    Path(faculty_id): Path<Uuid>,  
) -> Result<Json<Value>, StatusCode> {
    tracing::info!(
        "Admin access attempt: user_id={}, admin_level={:?}, user_faculty={:?}, requested_faculty={}",
        admin_user.session_user.user_id,
        admin_user.admin_role.admin_level,
        admin_user.session_user.faculty_id, 
        faculty_id
    );
    
    validate_faculty_access(&admin_user.session_user, faculty_id)?;
    // ... rest of handler
}
```

This enhanced faculty scope authorization system provides robust, secure, and flexible access control for faculty-scoped operations while maintaining good performance and developer experience.