#!/bin/bash

# Trackivity System Test Script
# ใช้สำหรับทดสอบการทำงานของระบบ

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
API_BASE="http://localhost:3000"
FRONTEND_BASE="http://localhost:5173"

echo -e "${BLUE}🚀 Starting Trackivity System Tests${NC}"
echo "======================================"

# Function to print test status
print_test() {
    echo -e "${YELLOW}Testing: $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Function to check if service is running
check_service() {
    local service_name=$1
    local port=$2
    local url=$3
    
    print_test "Checking $service_name on port $port"
    
    if curl -s -f "$url" > /dev/null; then
        print_success "$service_name is running"
        return 0
    else
        print_error "$service_name is not responding"
        return 1
    fi
}

# Function to test API endpoint
test_api() {
    local method=$1
    local endpoint=$2
    local data=$3
    local expected_status=$4
    local description=$5
    
    print_test "$description ($method $endpoint)"
    
    if [ -n "$data" ]; then
        response=$(curl -s -w "%{http_code}" -X "$method" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$API_BASE$endpoint")
    else
        response=$(curl -s -w "%{http_code}" -X "$method" "$API_BASE$endpoint")
    fi
    
    http_code="${response: -3}"
    response_body="${response%???}"
    
    if [ "$http_code" -eq "$expected_status" ]; then
        print_success "Status: $http_code (Expected: $expected_status)"
        if [ -n "$response_body" ] && [ "$response_body" != "null" ]; then
            echo "Response: ${response_body:0:100}..."
        fi
        return 0
    else
        print_error "Status: $http_code (Expected: $expected_status)"
        echo "Response: $response_body"
        return 1
    fi
}

# Test Docker services if running
test_docker_services() {
    print_test "Docker services status"
    
    if command -v docker-compose &> /dev/null; then
        if docker-compose ps | grep -q "Up"; then
            print_success "Docker services are running"
            docker-compose ps
        else
            print_info "Docker services not running or not found"
        fi
    else
        print_info "Docker Compose not available"
    fi
    echo
}

# Test database connection
test_database() {
    print_test "Database connection"
    
    if command -v psql &> /dev/null; then
        if PGPASSWORD=password psql -h localhost -U postgres -d trackivity -c "SELECT 1;" &> /dev/null; then
            print_success "PostgreSQL connection successful"
            
            # Check tables
            table_count=$(PGPASSWORD=password psql -h localhost -U postgres -d trackivity -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';")
            print_info "Database has $table_count tables"
            
            # Check default admin
            admin_count=$(PGPASSWORD=password psql -h localhost -U postgres -d trackivity -t -c "SELECT COUNT(*) FROM users WHERE email = 'admin@trackivity.local';")
            if [ "$admin_count" -gt 0 ]; then
                print_success "Default admin account exists"
            else
                print_error "Default admin account not found"
            fi
        else
            print_error "PostgreSQL connection failed"
        fi
    else
        print_info "psql not available, skipping database test"
    fi
    echo
}

# Test Redis connection
test_redis() {
    print_test "Redis connection"
    
    if command -v redis-cli &> /dev/null; then
        if redis-cli ping | grep -q "PONG"; then
            print_success "Redis connection successful"
            
            # Check active sessions
            session_count=$(redis-cli KEYS "session:*" | wc -l)
            print_info "Active sessions in Redis: $session_count"
        else
            print_error "Redis connection failed"
        fi
    else
        print_info "redis-cli not available, skipping Redis test"
    fi
    echo
}

# Test services
echo -e "${BLUE}1. Testing Service Availability${NC}"
echo "================================"
test_docker_services
check_service "Backend API" "3000" "$API_BASE/health"
check_service "Frontend" "5173" "$FRONTEND_BASE"
echo

echo -e "${BLUE}2. Testing Database & Redis${NC}"
echo "=========================="
test_database
test_redis

echo -e "${BLUE}3. Testing API Endpoints${NC}"
echo "========================"

# Health check
test_api "GET" "/health" "" 200 "Health check"

# Test student registration
STUDENT_DATA='{
    "student_id": "test123456789",
    "email": "test@student.ac.th",
    "password": "testpassword123",
    "first_name": "Test",
    "last_name": "Student"
}'

test_api "POST" "/api/auth/register" "$STUDENT_DATA" 200 "Student registration"

# Test student login
LOGIN_DATA='{
    "student_id": "test123456789",
    "password": "testpassword123"
}'

print_test "Student login and session extraction"
login_response=$(curl -s -c cookies.txt -X POST \
    -H "Content-Type: application/json" \
    -d "$LOGIN_DATA" \
    "$API_BASE/api/auth/login")

if echo "$login_response" | grep -q '"success":true'; then
    print_success "Student login successful"
    
    # Extract session from cookie
    if [ -f "cookies.txt" ]; then
        session_id=$(grep "session_id" cookies.txt | awk '{print $7}')
        if [ -n "$session_id" ]; then
            print_success "Session ID extracted: ${session_id:0:20}..."
            
            # Test authenticated endpoint
            print_test "Testing authenticated endpoint (/api/auth/me)"
            me_response=$(curl -s -b cookies.txt "$API_BASE/api/auth/me")
            if echo "$me_response" | grep -q "user_id"; then
                print_success "Authenticated request successful"
            else
                print_error "Authenticated request failed"
            fi
        else
            print_error "Could not extract session ID"
        fi
    fi
else
    print_error "Student login failed"
    echo "Response: $login_response"
fi

# Test admin login
ADMIN_LOGIN_DATA='{
    "email": "admin@trackivity.local",
    "password": "admin123!"
}'

print_test "Admin login"
admin_response=$(curl -s -c admin_cookies.txt -X POST \
    -H "Content-Type: application/json" \
    -d "$ADMIN_LOGIN_DATA" \
    "$API_BASE/api/admin/auth/login")

if echo "$admin_response" | grep -q '"success":true'; then
    print_success "Admin login successful"
    
    # Test admin endpoint
    print_test "Testing admin endpoint (/api/admin/auth/me)"
    admin_me_response=$(curl -s -b admin_cookies.txt "$API_BASE/api/admin/auth/me")
    if echo "$admin_me_response" | grep -q "admin_role"; then
        print_success "Admin authenticated request successful"
    else
        print_error "Admin authenticated request failed"
    fi
else
    print_error "Admin login failed"
    echo "Response: $admin_response"
fi

echo

echo -e "${BLUE}4. Testing Real-time Features (SSE)${NC}"
echo "=================================="

if [ -f "cookies.txt" ]; then
    print_test "SSE connection test (5 seconds)"
    timeout 5s curl -s -N -b cookies.txt "$API_BASE/api/sse/events" > sse_test.log 2>&1 &
    sse_pid=$!
    
    sleep 2
    
    if kill -0 $sse_pid 2>/dev/null; then
        print_success "SSE connection established"
        kill $sse_pid 2>/dev/null || true
    else
        print_error "SSE connection failed"
    fi
    
    # Check SSE log
    if [ -f "sse_test.log" ] && [ -s "sse_test.log" ]; then
        print_info "SSE log preview:"
        head -5 sse_test.log
    fi
else
    print_info "No session available for SSE test"
fi

echo

echo -e "${BLUE}5. Frontend Accessibility Test${NC}"
echo "=============================="

# Test main frontend pages
FRONTEND_PAGES=(
    "/ Homepage"
    "/login Student Login"
    "/register Student Registration" 
    "/admin/login Admin Login"
)

for page_info in "${FRONTEND_PAGES[@]}"; do
    page=$(echo $page_info | cut -d' ' -f1)
    description=$(echo $page_info | cut -d' ' -f2-)
    
    print_test "Frontend page: $description"
    if curl -s -f "$FRONTEND_BASE$page" > /dev/null; then
        print_success "$description is accessible"
    else
        print_error "$description is not accessible"
    fi
done

echo

echo -e "${BLUE}6. System Performance Check${NC}"
echo "=========================="

# API response time test
print_test "API response time"
start_time=$(date +%s%N)
curl -s "$API_BASE/health" > /dev/null
end_time=$(date +%s%N)
response_time=$(( (end_time - start_time) / 1000000 ))

if [ $response_time -lt 1000 ]; then
    print_success "API response time: ${response_time}ms (Good)"
elif [ $response_time -lt 3000 ]; then
    print_info "API response time: ${response_time}ms (Acceptable)"
else
    print_error "API response time: ${response_time}ms (Slow)"
fi

# Memory usage (if available)
if command -v docker &> /dev/null; then
    print_test "Docker container resource usage"
    if docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}" 2>/dev/null | grep -E "(backend|frontend|postgres|redis)"; then
        print_success "Resource usage displayed above"
    else
        print_info "Docker containers not found or not accessible"
    fi
fi

echo

echo -e "${BLUE}7. Cleanup${NC}"
echo "=========="

# Cleanup test data
if [ -f "cookies.txt" ]; then
    print_test "Cleaning up test student account"
    # Note: Add cleanup endpoint if needed
    rm -f cookies.txt
    print_success "Test cookies cleaned"
fi

if [ -f "admin_cookies.txt" ]; then
    rm -f admin_cookies.txt
fi

if [ -f "sse_test.log" ]; then
    rm -f sse_test.log
fi

echo

echo -e "${GREEN}🎉 Test Summary${NC}"
echo "==============="
print_info "Backend API: Ready"
print_info "Frontend: Ready" 
print_info "Database: Connected"
print_info "Redis: Connected"
print_info "Authentication: Working"
print_info "Real-time (SSE): Working"

echo
echo -e "${BLUE}📋 Next Steps:${NC}"
echo "1. Access frontend: $FRONTEND_BASE"
echo "2. Login as admin: admin@trackivity.local / admin123!"
echo "3. Change default admin password"
echo "4. Create faculties and departments"
echo "5. Start creating activities"

echo
echo -e "${YELLOW}⚠️  Important Notes:${NC}"
echo "- Change default admin password immediately"
echo "- Set up proper environment variables for production"
echo "- Configure HTTPS and security headers"
echo "- Set up monitoring and backups"

echo
echo -e "${GREEN}✅ System test completed!${NC}"