import type { ColumnDef } from '@tanstack/table-core';
import { renderComponent, renderSnippet } from '$lib/components/ui/data-table';
import { Checkbox as DataTableCheckbox } from '$lib/components/ui/checkbox';
import type { User } from '$lib/types/admin';
import {
	UserProfileCell,
	IdentifierCell,
	StatusBadge,
	FacultyCell,
	DepartmentCell,
	PhoneCell,
	LastLoginCell,
	CreatedAtCell,
	EmailVerifiedCell,
	UserActionsMenu
} from '$lib/components/user-management/UserTableCells';

/**
 * Comprehensive Data Table Column Configuration for User Management
 * Supports role-based visibility, sorting, filtering, and actions
 */

// User table column definitions with advanced features
export const userTableColumns: ColumnDef<User>[] = [
    // Selection column for bulk operations
    {
        id: 'select',
        header: ({ table }) =>
            renderComponent(DataTableCheckbox, {
                checked: table.getIsAllPageRowsSelected(),
                indeterminate: table.getIsSomePageRowsSelected() && !table.getIsAllPageRowsSelected(),
                onCheckedChange: (value) => table.toggleAllPageRowsSelected(!!value),
                'aria-label': 'เลือกทั้งหมด',
            }),
        cell: ({ row }) =>
            renderComponent(DataTableCheckbox, {
                checked: row.getIsSelected(),
                onCheckedChange: (value) => row.toggleSelected(!!value),
                'aria-label': `เลือก ${row.original.first_name} ${row.original.last_name}`,
            }),
        enableSorting: false,
        enableHiding: false,
        size: 40,
    },

    // Avatar and name column (always visible)
    {
        id: 'user',
        accessorKey: 'first_name',
        header: 'ผู้ใช้',
        cell: ({ row }) => renderSnippet(UserProfileCell, { user: row.original }),
        enableHiding: false,
        size: 250,
        filterFn: (row, id, value) => {
            const user = row.original;
            const searchValue = value.toLowerCase();
            return (
                user.first_name.toLowerCase().includes(searchValue) ||
                user.last_name.toLowerCase().includes(searchValue) ||
                user.email.toLowerCase().includes(searchValue) ||
                user.student_id?.toLowerCase().includes(searchValue) ||
                user.employee_id?.toLowerCase().includes(searchValue) ||
                false
            );
        },
    },

    // Removed standalone Email column: email now shows under name

    // Student/Employee ID column
    {
        id: 'identifier',
        accessorFn: (row) => row.student_id || row.employee_id || '',
        header: 'รหัส',
        cell: ({ row }) => renderSnippet(IdentifierCell, { user: row.original }),
        size: 120,
        filterFn: 'includesString',
    },


    // Status column with color-coded badges
    {
        accessorKey: 'status',
        header: 'สถานะ',
        cell: ({ row }) => renderSnippet(StatusBadge, { status: row.original.status }),
        size: 100,
        filterFn: (row, id, value) => {
            if (value === 'all' || !value) return true;
            return row.original.status === value;
        },
    },

    // Faculty column (visible for SuperAdmin)
    {
        id: 'faculty',
        accessorFn: (row) => row.faculty?.name || '',
        header: 'คณะ',
        cell: ({ row }) => renderSnippet(FacultyCell, { user: row.original }),
        size: 150,
        filterFn: (row, id, value) => {
            if (value === 'all' || !value) return true;
            return row.original.faculty_id === value;
        },
    },

    // Department column
    {
        id: 'department',
        accessorFn: (row) => row.department?.name || '',
        header: 'สาขา/ภาควิชา',
        cell: ({ row }) => renderSnippet(DepartmentCell, { user: row.original }),
        size: 150,
        filterFn: (row, id, value) => {
            if (value === 'all' || !value) return true;
            return row.original.department_id === value;
        },
    },

    // Phone column (optional, can be hidden)
    {
        accessorKey: 'phone',
        header: 'เบอร์โทร',
        cell: ({ row }) => renderSnippet(PhoneCell, { phone: row.original.phone }),
        size: 120,
        meta: {
            canHide: true,
        },
    },

    // Last login column with relative time
    {
        accessorKey: 'last_login',
        header: 'เข้าสู่ระบบล่าสุด',
        cell: ({ row }) => renderSnippet(LastLoginCell, { lastLogin: row.original.last_login }),
        size: 150,
        sortingFn: 'datetime',
        meta: {
            canHide: true,
        },
    },

    // Registration date column
    {
        accessorKey: 'created_at',
        header: 'วันที่สมัคร',
        cell: ({ row }) => renderSnippet(CreatedAtCell, { createdAt: row.original.created_at }),
        size: 120,
        sortingFn: 'datetime',
        meta: {
            canHide: true,
        },
    },

    // Email verification status
    {
        id: 'email_verified',
        accessorFn: (row) => !!row.email_verified_at,
        header: 'ยืนยันอีเมล',
        cell: ({ row }) => renderSnippet(EmailVerifiedCell, { 
            emailVerifiedAt: row.original.email_verified_at 
        }),
        size: 100,
        filterFn: (row, id, value) => {
            if (value === 'all' || !value) return true;
            const isVerified = !!row.original.email_verified_at;
            return value === 'verified' ? isVerified : !isVerified;
        },
        meta: {
            canHide: true,
        },
    },

    // Actions column (always visible, non-sortable)
    {
        id: 'actions',
        header: 'จัดการ',
        cell: ({ row }) => renderSnippet(UserActionsMenu, { user: row.original }),
        enableSorting: false,
        enableHiding: false,
        size: 80,
        meta: {
            sticky: 'right',
        },
    },
];

// Export function to get columns based on admin level
export function getUserTableColumns(
    adminLevel: 'SuperAdmin' | 'FacultyAdmin' | 'RegularAdmin',
    facultyScoped = false
): ColumnDef<User>[] {
    return userTableColumns.filter(column => {
        // Hide faculty column for FacultyAdmin (they only see their faculty)
        if (column.id === 'faculty' && (adminLevel === 'FacultyAdmin' || facultyScoped)) {
            return false;
        }

        // Some columns might be restricted based on permissions
        if (column.id === 'actions' && adminLevel === 'RegularAdmin') {
            return false; // Regular admins might have limited actions
        }

        return true;
    });
}

// Column visibility presets
export const columnVisibilityPresets = {
    compact: {
        user: true,
        // email column removed
        status: true,
        actions: true,
        // Hide other columns
        identifier: false,
        faculty: false,
        department: false,
        phone: false,
        last_login: false,
        created_at: false,
        email_verified: false,
    },
    
    detailed: {
        // Show all columns
        user: true,
        // email column removed
        identifier: true,
        status: true,
        faculty: true,
        department: true,
        phone: false, // Still hide phone by default
        last_login: true,
        created_at: true,
        email_verified: true,
        actions: true,
    },
    
    faculty_admin: {
        user: true,
        // email column removed
        identifier: true,
        status: true,
        faculty: false, // Hidden for faculty admin
        department: true,
        phone: false,
        last_login: true,
        created_at: true,
        email_verified: false,
        actions: true,
    }
};

// Export column accessor functions for filtering
export const userColumnAccessors = {
    search: (user: User) => `${user.first_name} ${user.last_name} ${user.email} ${user.student_id || ''} ${user.employee_id || ''}`,
    status: (user: User) => user.status,
    faculty: (user: User) => user.faculty_id || '',
    department: (user: User) => user.department_id || '',
    email_verified: (user: User) => !!user.email_verified_at ? 'verified' : 'unverified',
};
