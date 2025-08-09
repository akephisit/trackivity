#!/bin/bash

# Bootstrap Super Admin Script
# This script creates the initial super admin account for the Trackivity system
# Run this script only once when setting up the system for the first time

echo "Creating super admin account..."

curl -X POST http://localhost:3000/api/admin/bootstrap \
    -H "Content-Type: application/json" \
    -d '{
      "student_id": "ADMIN001",
      "email": "admin@trackivity.local", 
      "password": "admin123",
      "first_name": "System",
      "last_name": "Administrator"
    }'

echo ""
echo "Bootstrap completed!"
echo ""
echo "Default super admin credentials:"
echo "Email: admin@trackivity.local"
echo "Password: admin123!"
echo ""
echo "⚠️  IMPORTANT: Change the password immediately after first login!"