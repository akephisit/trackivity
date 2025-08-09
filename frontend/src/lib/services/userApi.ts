import type { 
    User, 
    UserFilter, 
    UserListResponse, 
    UserStats, 
    UserActivity, 
    UserUpdateRequest, 
    BulkUserOperation, 
    UserExportOptions,
    ApiResponse 
} from '$lib/types/admin';

/**
 * TypeScript API Client for Faculty-Scoped User Management
 * Handles role-based access control for SuperAdmin and FacultyAdmin
 */
export class UserManagementAPI {
    private baseUrl = '/api';

    /**
     * Get users with faculty-scoped access control
     * - SuperAdmin: All users system-wide with optional faculty filtering
     * - FacultyAdmin: Only users within their faculty
     */
    async getUsers(
        filters: UserFilter = {},
        page = 1,
        limit = 20
    ): Promise<UserListResponse> {
        try {
            const params = new URLSearchParams({
                page: page.toString(),
                limit: limit.toString(),
                ...Object.fromEntries(
                    Object.entries(filters).filter(([_, value]) => value !== undefined && value !== '')
                )
            });

            const response = await fetch(`${this.baseUrl}/users?${params}`, {
                method: 'GET',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                }
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data: ApiResponse<UserListResponse> = await response.json();
            
            if (!data.success || !data.data) {
                throw new Error(data.message || 'Failed to fetch users');
            }

            return data.data;
        } catch (error) {
            console.error('Failed to fetch users:', error);
            throw error;
        }
    }

    /**
     * Get faculty users (FacultyAdmin specific endpoint)
     */
    async getFacultyUsers(
        facultyId: string,
        filters: UserFilter = {},
        page = 1,
        limit = 20
    ): Promise<UserListResponse> {
        try {
            const params = new URLSearchParams({
                page: page.toString(),
                limit: limit.toString(),
                ...Object.fromEntries(
                    Object.entries(filters).filter(([_, value]) => value !== undefined && value !== '')
                )
            });

            const response = await fetch(`${this.baseUrl}/faculties/${facultyId}/users?${params}`, {
                method: 'GET',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                }
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data: ApiResponse<UserListResponse> = await response.json();
            
            if (!data.success || !data.data) {
                throw new Error(data.message || 'Failed to fetch faculty users');
            }

            return data.data;
        } catch (error) {
            console.error('Failed to fetch faculty users:', error);
            throw error;
        }
    }

    /**
     * Get system-wide users (SuperAdmin only)
     */
    async getAllUsers(
        filters: UserFilter = {},
        page = 1,
        limit = 20
    ): Promise<UserListResponse> {
        try {
            const params = new URLSearchParams({
                page: page.toString(),
                limit: limit.toString(),
                ...Object.fromEntries(
                    Object.entries(filters).filter(([_, value]) => value !== undefined && value !== '')
                )
            });

            const response = await fetch(`${this.baseUrl}/admin/users?${params}`, {
                method: 'GET',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                }
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data: ApiResponse<UserListResponse> = await response.json();
            
            if (!data.success || !data.data) {
                throw new Error(data.message || 'Failed to fetch all users');
            }

            return data.data;
        } catch (error) {
            console.error('Failed to fetch all users:', error);
            throw error;
        }
    }

    /**
     * Get user statistics with role-based scoping
     */
    async getUserStats(facultyId?: string): Promise<UserStats> {
        try {
            let endpoint = `${this.baseUrl}/users/stats`;
            if (facultyId) {
                endpoint = `${this.baseUrl}/faculties/${facultyId}/users/stats`;
            }

            const response = await fetch(endpoint, {
                method: 'GET',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                }
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data: ApiResponse<UserStats> = await response.json();
            
            if (!data.success || !data.data) {
                throw new Error(data.message || 'Failed to fetch user statistics');
            }

            return data.data;
        } catch (error) {
            console.error('Failed to fetch user statistics:', error);
            throw error;
        }
    }

    /**
     * Get user by ID
     */
    async getUser(userId: string): Promise<User> {
        try {
            const response = await fetch(`${this.baseUrl}/users/${userId}`, {
                method: 'GET',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                }
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data: ApiResponse<User> = await response.json();
            
            if (!data.success || !data.data) {
                throw new Error(data.message || 'Failed to fetch user');
            }

            return data.data;
        } catch (error) {
            console.error('Failed to fetch user:', error);
            throw error;
        }
    }

    /**
     * Update user information
     */
    async updateUser(userId: string, updates: UserUpdateRequest): Promise<User> {
        try {
            const response = await fetch(`${this.baseUrl}/users/${userId}`, {
                method: 'PATCH',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(updates)
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data: ApiResponse<User> = await response.json();
            
            if (!data.success || !data.data) {
                throw new Error(data.message || 'Failed to update user');
            }

            return data.data;
        } catch (error) {
            console.error('Failed to update user:', error);
            throw error;
        }
    }

    /**
     * Update user status (activate/deactivate/suspend)
     */
    async updateUserStatus(userId: string, status: 'active' | 'inactive' | 'suspended'): Promise<User> {
        try {
            const response = await fetch(`${this.baseUrl}/users/${userId}/status`, {
                method: 'PATCH',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ status })
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data: ApiResponse<User> = await response.json();
            
            if (!data.success || !data.data) {
                throw new Error(data.message || 'Failed to update user status');
            }

            return data.data;
        } catch (error) {
            console.error('Failed to update user status:', error);
            throw error;
        }
    }

    /**
     * Update user role
     */
    async updateUserRole(userId: string, role: 'student' | 'faculty' | 'staff' | 'admin'): Promise<User> {
        try {
            const response = await fetch(`${this.baseUrl}/users/${userId}/role`, {
                method: 'PATCH',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ role })
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data: ApiResponse<User> = await response.json();
            
            if (!data.success || !data.data) {
                throw new Error(data.message || 'Failed to update user role');
            }

            return data.data;
        } catch (error) {
            console.error('Failed to update user role:', error);
            throw error;
        }
    }

    /**
     * Transfer user to different faculty/department (SuperAdmin only)
     */
    async transferUser(
        userId: string, 
        facultyId?: string, 
        departmentId?: string
    ): Promise<User> {
        try {
            const response = await fetch(`${this.baseUrl}/users/${userId}/transfer`, {
                method: 'PATCH',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ faculty_id: facultyId, department_id: departmentId })
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data: ApiResponse<User> = await response.json();
            
            if (!data.success || !data.data) {
                throw new Error(data.message || 'Failed to transfer user');
            }

            return data.data;
        } catch (error) {
            console.error('Failed to transfer user:', error);
            throw error;
        }
    }

    /**
     * Get user activity history
     */
    async getUserActivity(userId: string, limit = 20): Promise<UserActivity[]> {
        try {
            const params = new URLSearchParams({
                limit: limit.toString()
            });

            const response = await fetch(`${this.baseUrl}/users/${userId}/activity?${params}`, {
                method: 'GET',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                }
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data: ApiResponse<UserActivity[]> = await response.json();
            
            if (!data.success || !data.data) {
                throw new Error(data.message || 'Failed to fetch user activity');
            }

            return data.data;
        } catch (error) {
            console.error('Failed to fetch user activity:', error);
            throw error;
        }
    }

    /**
     * Perform bulk operations on users
     */
    async bulkOperateUsers(operation: BulkUserOperation): Promise<{ success: number; failed: number; errors?: string[] }> {
        try {
            const response = await fetch(`${this.baseUrl}/users/bulk`, {
                method: 'POST',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(operation)
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data: ApiResponse<{ success: number; failed: number; errors?: string[] }> = await response.json();
            
            if (!data.success || !data.data) {
                throw new Error(data.message || 'Failed to perform bulk operation');
            }

            return data.data;
        } catch (error) {
            console.error('Failed to perform bulk operation:', error);
            throw error;
        }
    }

    /**
     * Export user data
     */
    async exportUsers(options: UserExportOptions): Promise<Blob> {
        try {
            const response = await fetch(`${this.baseUrl}/users/export`, {
                method: 'POST',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(options)
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            return await response.blob();
        } catch (error) {
            console.error('Failed to export users:', error);
            throw error;
        }
    }

    /**
     * Delete user (soft delete)
     */
    async deleteUser(userId: string): Promise<void> {
        try {
            const response = await fetch(`${this.baseUrl}/users/${userId}`, {
                method: 'DELETE',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                }
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data: ApiResponse = await response.json();
            
            if (!data.success) {
                throw new Error(data.message || 'Failed to delete user');
            }
        } catch (error) {
            console.error('Failed to delete user:', error);
            throw error;
        }
    }

    /**
     * Restore deleted user
     */
    async restoreUser(userId: string): Promise<User> {
        try {
            const response = await fetch(`${this.baseUrl}/users/${userId}/restore`, {
                method: 'POST',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                }
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data: ApiResponse<User> = await response.json();
            
            if (!data.success || !data.data) {
                throw new Error(data.message || 'Failed to restore user');
            }

            return data.data;
        } catch (error) {
            console.error('Failed to restore user:', error);
            throw error;
        }
    }
}

// Singleton instance
export const userApi = new UserManagementAPI();