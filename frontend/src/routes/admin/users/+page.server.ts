import { requireAdmin } from '$lib/server/auth';
import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { 
    User, 
    UserFilter, 
    UserStats,
    Faculty
} from '$lib/types/admin';
import { AdminLevel } from '$lib/types/admin';
import { PUBLIC_API_URL } from '$env/static/public';

/**
 * Server Load Function for General User Management
 * Implements role-based access control:
 * - SuperAdmin: Can view all users system-wide
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
            // SuperAdmin can view all users via local proxy
            apiEndpoint = `/api/admin/system-users`;
            statsEndpoint = `/api/admin/user-statistics`;
        } else if (adminLevel === AdminLevel.FacultyAdmin) {
            // FacultyAdmin can only view users within their faculty
            if (!facultyId) {
                throw error(403, 'Faculty admin must be associated with a faculty');
            }
            apiEndpoint = `/api/faculties/${facultyId}/users`;
            statsEndpoint = `/api/faculties/${facultyId}/users/stats`;
        } else {
            throw error(403, 'Insufficient permissions to view user data');
        }

        // Build query string for API request
        const queryParams = new URLSearchParams();
        if (filters.search) queryParams.set('search', filters.search);
        if (filters.faculty_id) queryParams.set('faculty_id', filters.faculty_id);
        if (filters.department_id) queryParams.set('department_id', filters.department_id);
        if (filters.status && filters.status !== 'all') queryParams.set('status', filters.status);
        if (filters.role && filters.role !== 'all') queryParams.set('role', filters.role);
        if (filters.created_after) queryParams.set('created_after', filters.created_after);
        if (filters.created_before) queryParams.set('created_before', filters.created_before);
        // Use limit/offset as expected by backend
        queryParams.set('limit', limit.toString());
        queryParams.set('offset', offset.toString());

        // Make API requests
        const [usersResponse, statsResponse, facultiesResponse] = await Promise.all([
            event.fetch(`${apiEndpoint}?${queryParams.toString()}`),
            event.fetch(statsEndpoint),
            // Load faculties for filtering (only for SuperAdmin)
            adminLevel === AdminLevel.SuperAdmin ? event.fetch(`/api/faculties`) : Promise.resolve(null)
        ]);

        // Process users response
        let users: User[] = [];
        let pagination = { page: page, total_pages: 1, total_count: 0, limit: limit } as any;

        if (usersResponse.ok) {
            const usersResult = await usersResponse.json();
            const src = (usersResult.data || usersResult) as any;
            const rawUsers: any[] = src.users || src.data?.users || [];
            const totalCount: number = src.total_count ?? src.pagination?.total ?? rawUsers.length;

            users = rawUsers.map((u: any) => {
                // Determine user status similar to admin/admins page
                const isEnabled = u.is_enabled !== undefined ? Boolean(u.is_enabled) : true;
                const isActive = u.is_active !== undefined && u.is_active !== null ? Boolean(u.is_active) : false;
                
                let status: User['status'];
                if (!isEnabled) {
                    status = 'disabled';
                } else if (isActive) {
                    status = 'online';
                } else {
                    status = 'offline';
                }
                const lastLogin = u.last_login ? new Date(u.last_login).toISOString() : undefined;
                const department = u.department_name ? { id: u.department_id, name: u.department_name } : u.department;
                const faculty = u.faculty_name ? { id: u.faculty_id, name: u.faculty_name } : u.faculty;
                // Determine user role based on admin_role and user data
                let role: User['role'] = 'student'; // default
                
                if (u.admin_role) {
                    // If user has admin role, determine the specific admin type
                    const adminLevel = u.admin_role.admin_level;
                    switch (adminLevel) {
                        case 'SuperAdmin':
                            role = 'super_admin';
                            break;
                        case 'FacultyAdmin':
                            role = 'faculty_admin';
                            break;
                        case 'RegularAdmin':
                            role = 'regular_admin';
                            break;
                        default:
                            role = 'admin';
                    }
                } else {
                    role = u.role || 'student';
                }
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

            pagination = {
                page,
                limit,
                total_count: totalCount,
                total_pages: Math.max(1, Math.ceil(totalCount / limit))
            };
        } else {
            console.error('Failed to load users:', await usersResponse.text());
        }

        // Process stats response
        let stats: UserStats = {
            total_users: 0,
            active_users: 0,
            inactive_users: 0,
            students: 0,
            faculty: 0,
            staff: 0,
            recent_registrations: 0
        };

        if (statsResponse.ok) {
            const statsResult = await statsResponse.json();
            const rawStats = statsResult.data || statsResult;
            // Support both system-wide and faculty-scoped shapes
            if (rawStats.system_stats) {
                stats = {
                    total_users: rawStats.system_stats.total_users || 0,
                    active_users: rawStats.system_stats.active_users_30_days || 0,
                    inactive_users: Math.max(0, (rawStats.system_stats.total_users || 0) - (rawStats.system_stats.active_users_30_days || 0)),
                    students: rawStats.system_stats.total_users || 0,
                    faculty: 0,
                    staff: 0,
                    recent_registrations: rawStats.system_stats.new_users_30_days || 0,
                } as UserStats;
            } else if (rawStats.total_users !== undefined) {
                stats = rawStats as UserStats;
            }
        } else {
            console.error('Failed to load user stats:', await statsResponse.text());
        }

        // Process faculties response (for SuperAdmin only)
        let faculties: Faculty[] = [];
        if (facultiesResponse && facultiesResponse.ok) {
            const facultiesResult = await facultiesResponse.json();
            const rawFaculties = facultiesResult.data || facultiesResult;
            faculties = rawFaculties?.faculties || rawFaculties || [];
        }

        return {
            users,
            stats,
            faculties,
            pagination,
            filters,
            adminLevel,
            facultyId,
            canManageAllUsers: adminLevel === AdminLevel.SuperAdmin
        };

    } catch (err) {
        console.error('Error in users page load:', err);
        throw error(500, 'Failed to load user data');
    }
};
