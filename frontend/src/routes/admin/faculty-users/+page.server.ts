import { requireAdmin } from '$lib/server/auth';
import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { 
    User, 
    UserFilter, 
    UserListResponse, 
    UserStats,
    Faculty,
    Department,
    AdminLevel
} from '$lib/types/admin';

/**
 * Server Load Function for Faculty-Scoped User Management
 * Implements role-based access control:
 * - SuperAdmin: Can view all users system-wide with optional faculty filtering
 * - FacultyAdmin: Can only view users within their faculty
 */
export const load: PageServerLoad = async (event) => {
    // Ensure user is authenticated as admin
    const user = await requireAdmin(event);
    const adminLevel = user.admin_role?.admin_level;
    const facultyId = user.admin_role?.faculty_id;

    // Extract query parameters for filtering and pagination
    const url = event.url;
    const searchParams = url.searchParams;
    
    const filters: UserFilter = {
        search: searchParams.get('search') || undefined,
        faculty_id: searchParams.get('faculty_id') || undefined,
        department_id: searchParams.get('department_id') || undefined,
        status: (searchParams.get('status') as any) || 'all',
        role: (searchParams.get('role') as any) || 'all',
        created_after: searchParams.get('created_after') || undefined,
        created_before: searchParams.get('created_before') || undefined
    };

    const page = parseInt(searchParams.get('page') || '1');
    const limit = parseInt(searchParams.get('limit') || '20');

    try {
        // Determine API endpoint based on admin level
        let apiEndpoint: string;
        let statsEndpoint: string;

        if (adminLevel === AdminLevel.SuperAdmin) {
            // SuperAdmin can view all users or filter by faculty
            apiEndpoint = filters.faculty_id 
                ? `/api/faculties/${filters.faculty_id}/users`
                : '/api/admin/users';
            statsEndpoint = filters.faculty_id 
                ? `/api/faculties/${filters.faculty_id}/users/stats`
                : '/api/users/stats';
        } else if (adminLevel === AdminLevel.FacultyAdmin && facultyId) {
            // FacultyAdmin is scoped to their faculty only
            apiEndpoint = `/api/faculties/${facultyId}/users`;
            statsEndpoint = `/api/faculties/${facultyId}/users/stats`;
            
            // Override any faculty_id filter to ensure scoping
            filters.faculty_id = facultyId;
        } else {
            error(403, 'Insufficient permissions to access user management');
        }

        // Build query parameters
        const params = new URLSearchParams({
            page: page.toString(),
            limit: limit.toString(),
            ...Object.fromEntries(
                Object.entries(filters).filter(([_, value]) => value !== undefined && value !== '' && value !== 'all')
            )
        });

        // Parallel fetch of users and statistics
        const [usersResponse, statsResponse, facultiesResponse, departmentsResponse] = await Promise.all([
            // Fetch users
            event.fetch(`${apiEndpoint}?${params}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json'
                }
            }),
            
            // Fetch statistics
            event.fetch(statsEndpoint, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json'
                }
            }),

            // Fetch faculties for SuperAdmin filtering
            adminLevel === AdminLevel.SuperAdmin ? 
                event.fetch('/api/faculties', {
                    method: 'GET',
                    headers: {
                        'Content-Type': 'application/json'
                    }
                }) : Promise.resolve(null),

            // Fetch departments for the relevant faculty
            event.fetch(`/api/faculties/${facultyId || filters.faculty_id || 'current'}/departments`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json'
                }
            }).catch(() => null) // Handle case where faculty doesn't have departments endpoint
        ]);

        // Process users response
        if (!usersResponse.ok) {
            console.error('Failed to fetch users:', usersResponse.status, usersResponse.statusText);
            error(usersResponse.status, 'Failed to fetch users');
        }

        const usersData = await usersResponse.json();
        if (!usersData.success) {
            error(500, usersData.message || 'Failed to fetch users');
        }

        // Process statistics response
        if (!statsResponse.ok) {
            console.error('Failed to fetch user statistics:', statsResponse.status, statsResponse.statusText);
            error(statsResponse.status, 'Failed to fetch user statistics');
        }

        const statsData = await statsResponse.json();
        if (!statsData.success) {
            error(500, statsData.message || 'Failed to fetch user statistics');
        }

        // Process faculties response (for SuperAdmin)
        let faculties: Faculty[] = [];
        if (facultiesResponse && facultiesResponse.ok) {
            const facultiesData = await facultiesResponse.json();
            if (facultiesData.success) {
                faculties = facultiesData.data || [];
            }
        }

        // Process departments response
        let departments: Department[] = [];
        if (departmentsResponse && departmentsResponse.ok) {
            const departmentsData = await departmentsResponse.json();
            if (departmentsData.success) {
                departments = departmentsData.data || [];
            }
        }

        // Return data to the page component
        return {
            users: usersData.data as UserListResponse,
            stats: statsData.data as UserStats,
            faculties,
            departments,
            filters,
            adminLevel,
            facultyId,
            pagination: {
                page,
                limit
            },
            // Pass current user info for permission checking
            currentUser: user,
            // SEO and meta data
            meta: {
                title: adminLevel === AdminLevel.SuperAdmin 
                    ? 'ระบบจัดการผู้ใช้ทั้งหมด'
                    : 'จัดการผู้ใช้คณะ',
                description: adminLevel === AdminLevel.SuperAdmin
                    ? 'จัดการผู้ใช้ทั้งระบบพร้อมการกรองตามคณะ'
                    : 'จัดการข้อมูลผู้ใช้ในคณะของคุณ'
            }
        };

    } catch (err) {
        console.error('Error in faculty-users page load:', err);
        
        // Provide user-friendly error messages
        if (err instanceof Error) {
            if (err.message.includes('fetch')) {
                error(503, 'ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้ กรุณาลองใหม่อีกครั้ง');
            }
            if (err.message.includes('unauthorized') || err.message.includes('403')) {
                error(403, 'คุณไม่มีสิทธิ์เข้าถึงหน้านี้');
            }
        }
        
        error(500, 'เกิดข้อผิดพลาดในการโหลดข้อมูล กรุณาลองใหม่อีกครั้ง');
    }
};