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

    try {
        // Determine API endpoint based on admin level
        let apiEndpoint: string;
        let statsEndpoint: string;

        if (adminLevel === AdminLevel.SuperAdmin) {
            // SuperAdmin can view all users
            apiEndpoint = '/api/admin/users';
            statsEndpoint = '/api/admin/users/stats';
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
        queryParams.set('page', page.toString());
        queryParams.set('limit', limit.toString());

        // Make API requests
        const [usersResponse, statsResponse, facultiesResponse] = await Promise.all([
            event.fetch(`${apiEndpoint}?${queryParams.toString()}`, {
                headers: {
                    'Cookie': event.request.headers.get('Cookie') || ''
                }
            }),
            event.fetch(statsEndpoint, {
                headers: {
                    'Cookie': event.request.headers.get('Cookie') || ''
                }
            }),
            // Load faculties for filtering (only for SuperAdmin)
            adminLevel === AdminLevel.SuperAdmin ? event.fetch('/api/faculties', {
                headers: {
                    'Cookie': event.request.headers.get('Cookie') || ''
                }
            }) : Promise.resolve(null)
        ]);

        // Process users response
        let users: User[] = [];
        let pagination = { page: 1, total_pages: 1, total_count: 0, limit: 20 };

        if (usersResponse.ok) {
            const usersResult = await usersResponse.json();
            users = usersResult.users || usersResult.data || [];
            if (usersResult.pagination) {
                pagination = {
                    page: usersResult.pagination.page,
                    total_pages: usersResult.pagination.pages,
                    total_count: usersResult.pagination.total,
                    limit: usersResult.pagination.limit
                };
            }
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
            stats = statsResult.data || stats;
        } else {
            console.error('Failed to load user stats:', await statsResponse.text());
        }

        // Process faculties response (for SuperAdmin only)
        let faculties: Faculty[] = [];
        if (facultiesResponse && facultiesResponse.ok) {
            const facultiesResult = await facultiesResponse.json();
            faculties = facultiesResult.data || [];
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