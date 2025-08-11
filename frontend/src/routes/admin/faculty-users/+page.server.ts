import { requireAdmin } from '$lib/server/auth';
import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { 
    User, 
    UserFilter, 
    UserListResponse, 
    UserStats,
    Faculty,
    Department
} from '$lib/types/admin';
import { AdminLevel } from '$lib/types/admin';

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
    const offset = (page - 1) * limit;

    try {
        // Determine API endpoint based on admin level
        let apiEndpoint: string;
        let statsEndpoint: string;

        if (adminLevel === AdminLevel.SuperAdmin) {
            // SuperAdmin can view all users or filter by faculty
            apiEndpoint = filters.faculty_id 
                ? `/api/faculties/${filters.faculty_id}/users`
                : '/api/admin/system-users';
            // Use existing proxy routes
            statsEndpoint = filters.faculty_id 
                ? `/api/faculties/${filters.faculty_id}/users/stats`
                : '/api/admin/user-statistics';
        } else if (adminLevel === AdminLevel.FacultyAdmin && facultyId) {
            // FacultyAdmin is scoped to their faculty only
            apiEndpoint = `/api/faculties/${facultyId}/users`;
            // Faculty-scoped stats via existing proxy route
            statsEndpoint = `/api/faculties/${facultyId}/users/stats`;
            
            // Override any faculty_id filter to ensure scoping
            filters.faculty_id = facultyId;
        } else {
            error(403, 'Insufficient permissions to access user management');
        }

        // Build query parameters (backend expects limit/offset)
        const params = new URLSearchParams({
            limit: limit.toString(),
            offset: offset.toString(),
        });
        for (const [k, v] of Object.entries(filters)) {
            if (v !== undefined && v !== '' && v !== 'all') params.set(k, String(v));
        }

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
        // Check for error status or missing data
        if (usersData.status === 'error' || (!usersData.data && !usersData.users)) {
            error(500, usersData.message || 'Failed to fetch users');
        }

        // Process statistics response
        if (!statsResponse.ok) {
            console.error('Failed to fetch user statistics:', statsResponse.status, statsResponse.statusText);
            error(statsResponse.status, 'Failed to fetch user statistics');
        }

        const statsData = await statsResponse.json();
        // Check for error status or missing data
        if (statsData.status === 'error' || !statsData.data) {
            error(500, statsData.message || 'Failed to fetch user statistics');
        }

        // Process faculties response (for SuperAdmin)
        let faculties: Faculty[] = [];
        if (facultiesResponse && facultiesResponse.ok) {
            const facultiesData = await facultiesResponse.json();
            if (facultiesData.status === 'success') {
                faculties = facultiesData.data || [];
            }
        }

        // Process departments response
        let departments: Department[] = [];
        if (departmentsResponse && departmentsResponse.ok) {
            const departmentsData = await departmentsResponse.json();
            if (departmentsData.status === 'success') {
                departments = departmentsData.data || [];
            }
        }

        // Normalize users into a consistent shape expected by the table
        const src = (usersData.data || usersData) as any;
        const rawUsers: any[] = src.users || src.data?.users || [];
        const totalCount: number = src.total_count ?? src.pagination?.total ?? rawUsers.length;

        const normalizedUsers: User[] = rawUsers.map((u) => {
            // Some endpoints return flat fields (faculty_name/department_name) and session info
            const lastLogin = u.last_login ? new Date(u.last_login).toISOString() : undefined;
            const isActive = typeof u.is_active === 'boolean' ? u.is_active : false;
            // Compose nested department/faculty for display (name used in cells)
            const department: any = u.department_name
                ? { id: u.department_id, name: u.department_name, code: u.department_code }
                : u.department || undefined;
            const faculty: any = u.faculty_name
                ? { id: u.faculty_id, name: u.faculty_name, code: u.faculty_code }
                : u.faculty || undefined;

            const role: User['role'] = u.admin_role ? 'admin' : (u.role || 'student');
            const status: User['status'] = isActive ? 'active' : 'inactive';

            return {
                id: u.id,
                email: u.email,
                first_name: u.first_name,
                last_name: u.last_name,
                student_id: u.student_id,
                employee_id: u.employee_id,
                department_id: u.department_id,
                faculty_id: u.faculty_id,
                status,
                role,
                phone: u.phone,
                avatar: u.avatar,
                last_login: lastLogin,
                email_verified_at: u.email_verified_at,
                created_at: u.created_at ? new Date(u.created_at).toISOString() : new Date().toISOString(),
                updated_at: u.updated_at ? new Date(u.updated_at).toISOString() : new Date().toISOString(),
                department,
                faculty,
            } as User;
        });

        // Normalize stats for SuperAdmin (system-wide) vs Faculty-scoped
        const rawStats = (statsData.data || statsData) as any;
        let normalizedStats: UserStats;
        if (rawStats && rawStats.system_stats) {
            normalizedStats = {
                total_users: rawStats.system_stats.total_users || 0,
                active_users: rawStats.system_stats.active_users_30_days || 0,
                inactive_users: Math.max(0, (rawStats.system_stats.total_users || 0) - (rawStats.system_stats.active_users_30_days || 0)),
                students: rawStats.system_stats.total_users || 0,
                faculty: 0,
                staff: 0,
                recent_registrations: rawStats.system_stats.new_users_30_days || 0,
                faculty_breakdown: Array.isArray(rawStats.faculty_stats)
                    ? rawStats.faculty_stats.map((f: any) => ({
                          faculty_id: f.faculty_id,
                          faculty_name: f.faculty_name,
                          user_count: f.total_users,
                      }))
                    : [],
            } as UserStats;
        } else {
            normalizedStats = rawStats as UserStats;
        }

        // Return data to the page component in unified format
        return {
            users: {
                users: normalizedUsers,
                pagination: {
                    page,
                    limit,
                    total: totalCount,
                    pages: Math.max(1, Math.ceil(totalCount / limit)),
                },
                filters,
                total_count: totalCount,
            } satisfies UserListResponse,
            stats: normalizedStats,
            faculties,
            departments,
            filters,
            adminLevel,
            facultyId,
            pagination: {
                page,
                limit,
                total: totalCount,
                pages: Math.max(1, Math.ceil(totalCount / limit)),
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
