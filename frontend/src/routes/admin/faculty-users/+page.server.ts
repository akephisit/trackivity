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
import { api } from '$lib/server/api-client';

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
        // Stats endpoints defined in backend routes:
        //  - /api/admin/user-statistics (SuperAdmin, system-wide)
        //  - /api/admin/faculty-user-statistics?faculty_id=... (SuperAdmin or FacultyAdmin for scoped faculty)
        let statsPath: string;
        let statsParams: Record<string, string> | undefined;

        if (adminLevel === AdminLevel.SuperAdmin) {
            // SuperAdmin can view all users or filter by faculty
            apiEndpoint = filters.faculty_id 
                ? `/api/faculties/${filters.faculty_id}/users`
                : '/api/admin/system-users';
            if (filters.faculty_id) {
                statsPath = '/api/admin/faculty-user-statistics';
                statsParams = { faculty_id: filters.faculty_id };
            } else {
                statsPath = '/api/admin/user-statistics';
            }
        } else if (adminLevel === AdminLevel.FacultyAdmin && facultyId) {
            // FacultyAdmin is scoped to their faculty only
            apiEndpoint = `/api/faculties/${facultyId}/users`;
            statsPath = '/api/admin/faculty-user-statistics';
            statsParams = { faculty_id: facultyId };
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

        // Convert params to object for API client
        const paramsObj: Record<string, string> = {};
        params.forEach((value, key) => {
            paramsObj[key] = value;
        });

        // Parallel fetch of users and statistics
        const [usersResponse, statsResponse, facultiesResponse, departmentsResponse] = await Promise.all([
            // Fetch users
            api.get(event, apiEndpoint, paramsObj, { throwOnHttpError: false }),
            
            // Fetch statistics
            api.get(event, statsPath, statsParams, { throwOnHttpError: false }),

            // Fetch faculties for SuperAdmin filtering
            adminLevel === AdminLevel.SuperAdmin ? 
                api.get(event, '/api/faculties', undefined, { throwOnHttpError: false }) : Promise.resolve(null),

            // Fetch departments for the relevant faculty
            (facultyId || filters.faculty_id)
                ? api.get(event, `/api/faculties/${facultyId || filters.faculty_id}/departments`, undefined, { throwOnHttpError: false })
                : Promise.resolve(null)
        ]);

        // Process users response
        if (!usersResponse || !usersResponse.success) {
            console.error('Failed to fetch users:', usersResponse.error);
            error(500, usersResponse.error || 'Failed to fetch users');
        }

        const usersData = usersResponse.data;
        // Check for missing data
        if (!usersData && !usersData?.users) {
            error(500, 'Failed to fetch users');
        }

        // Process statistics response
        let statsData = statsResponse?.data;
        if (!statsResponse?.success) {
            console.warn('Failed to fetch user statistics:', statsResponse?.error);
            statsData = undefined;
        }

        // Process faculties response (for SuperAdmin)
        let faculties: Faculty[] = [];
        if (facultiesResponse && facultiesResponse.success) {
            faculties = facultiesResponse.data || [];
        }

        // Process departments response
        let departments: Department[] = [];
        if (departmentsResponse && departmentsResponse.success) {
            departments = departmentsResponse.data || [];
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
            // Use admin-like semantics for status display
            const status: User['status'] = isActive ? 'online' : 'offline';

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
        const rawStats = (statsData && ((statsData as any).data || statsData)) as any;
        let normalizedStats: UserStats | undefined;
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
        } else if (rawStats) {
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
