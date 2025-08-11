/**
 * Role-based Navigation Routes Configuration
 * กำหนดเส้นทางและ permissions สำหรับระบบต่าง ๆ
 */

import type { RouteConfig, NavigationItem, Permission, AdminLevel } from '$lib/types';

// ===== ROUTE DEFINITIONS =====
export const ROUTES: RouteConfig[] = [
  // Public Routes
  {
    path: '/',
    title: 'หน้าแรก',
    permissions: [],
    admin_levels: []
  },
  {
    path: '/login',
    title: 'เข้าสู่ระบบ',
    permissions: [],
    admin_levels: []
  },
  {
    path: '/register',
    title: 'สมัครสมาชิก',
    permissions: [],
    admin_levels: []
  },
  {
    path: '/unauthorized',
    title: 'ไม่มีสิทธิ์เข้าถึง',
    permissions: [],
    admin_levels: []
  },

  // Student Routes
  {
    path: '/dashboard',
    title: 'แดชบอร์ด',
    permissions: ['ViewPersonalQR'],
    admin_levels: [],
    icon: 'dashboard'
  },
  {
    path: '/qr',
    title: 'QR Code ของฉัน',
    permissions: ['ViewPersonalQR'],
    admin_levels: [],
    icon: 'qr_code'
  },
  {
    path: '/activities',
    title: 'กิจกรรมของฉัน',
    permissions: ['ViewPersonalHistory'],
    admin_levels: [],
    icon: 'event'
  },
  {
    path: '/profile',
    title: 'โปรไฟล์',
    permissions: [],
    admin_levels: [],
    icon: 'person'
  },

  // Regular Admin Routes
  {
    path: '/admin/scanner',
    title: 'สแกน QR Code',
    permissions: ['ScanQRCodes'],
    admin_levels: ['RegularAdmin', 'FacultyAdmin', 'SuperAdmin'],
    icon: 'qr_code_scanner'
  },
  {
    path: '/admin/assigned-activities',
    title: 'กิจกรรมที่รับผิดชอบ',
    permissions: ['ViewAssignedActivities'],
    admin_levels: ['RegularAdmin', 'FacultyAdmin', 'SuperAdmin'],
    icon: 'assignment'
  },
  {
    path: '/admin/sessions',
    title: 'เซสชันของฉัน',
    permissions: ['ViewPersonalSessions'],
    admin_levels: ['RegularAdmin', 'FacultyAdmin', 'SuperAdmin'],
    icon: 'devices'
  },

  // Faculty Admin Routes
  {
    path: '/admin/faculty',
    title: 'จัดการคณะ',
    permissions: ['ViewFacultyUsers', 'ManageFacultyActivities'],
    admin_levels: ['FacultyAdmin', 'SuperAdmin'],
    icon: 'school',
    children: [
      {
        path: '/admin/faculty/dashboard',
        title: 'แดshboard คณะ',
        permissions: ['ViewFacultyAnalytics'],
        admin_levels: ['FacultyAdmin', 'SuperAdmin'],
        icon: 'analytics'
      },
      {
        path: '/admin/faculty/students',
        title: 'จัดการนักศึกษา',
        permissions: ['ViewFacultyUsers', 'CreateFacultyUsers'],
        admin_levels: ['FacultyAdmin', 'SuperAdmin'],
        icon: 'group'
      },
      {
        path: '/admin/faculty/activities',
        title: 'จัดการกิจกรรม',
        permissions: ['ManageFacultyActivities'],
        admin_levels: ['FacultyAdmin', 'SuperAdmin'],
        icon: 'event'
      },
      {
        path: '/admin/faculty/reports',
        title: 'รายงาน',
        permissions: ['ViewFacultyAnalytics'],
        admin_levels: ['FacultyAdmin', 'SuperAdmin'],
        icon: 'assessment'
      },
      {
        path: '/admin/faculty/sessions',
        title: 'จัดการเซสชัน',
        permissions: ['ViewFacultySessions', 'ManageFacultySessions'],
        admin_levels: ['FacultyAdmin', 'SuperAdmin'],
        icon: 'security'
      }
    ]
  },

  // Super Admin Routes
  {
    path: '/admin/system',
    title: 'จัดการระบบ',
    permissions: ['ViewSystemAnalytics'],
    admin_levels: ['SuperAdmin'],
    icon: 'admin_panel_settings',
    children: [
      {
        path: '/admin/system/dashboard',
        title: 'ภาพรวมระบบ',
        permissions: ['ViewSystemAnalytics'],
        admin_levels: ['SuperAdmin'],
        icon: 'analytics'
      },
      {
        path: '/admin/system/faculties',
        title: 'จัดการคณะ',
        permissions: ['ViewAllFaculties', 'CreateFaculties'],
        admin_levels: ['SuperAdmin'],
        icon: 'account_balance'
      },
      {
        path: '/admin/system/users',
        title: 'จัดการผู้ใช้',
        permissions: ['ViewAllUsers', 'CreateUsers'],
        admin_levels: ['SuperAdmin'],
        icon: 'people'
      },
      {
        path: '/admin/system/admins',
        title: 'จัดการผู้ดูแลระบบ',
        permissions: ['ViewAllUsers', 'CreateUsers'],
        admin_levels: ['SuperAdmin'],
        icon: 'admin_panel_settings'
      },
      {
        path: '/admin/system/sessions',
        title: 'จัดการเซสชัน',
        permissions: ['ViewAllSessions', 'ManageAllSessions'],
        admin_levels: ['SuperAdmin'],
        icon: 'security'
      },
      {
        path: '/admin/system/settings',
        title: 'ตั้งค่าระบบ',
        permissions: ['ViewSystemAnalytics'],
        admin_levels: ['SuperAdmin'],
        icon: 'settings'
      }
    ]
  }
];

// ===== NAVIGATION HELPERS =====
export function hasRoutePermission(
  route: RouteConfig,
  userPermissions: Permission[],
  adminLevel?: AdminLevel
): boolean {
  // Check admin level requirement
  if (route.admin_levels.length > 0 && !adminLevel) {
    return false;
  }

  if (route.admin_levels.length > 0 && !route.admin_levels.includes(adminLevel!)) {
    return false;
  }

  // Check permissions
  if (route.permissions.length === 0) {
    return true; // No permission required
  }

  return route.permissions.some(permission => userPermissions.includes(permission));
}

export function filterRoutesByPermissions(
  routes: RouteConfig[],
  userPermissions: Permission[],
  adminLevel?: AdminLevel
): RouteConfig[] {
  return routes
    .filter(route => hasRoutePermission(route, userPermissions, adminLevel))
    .map(route => ({
      ...route,
      children: route.children 
        ? filterRoutesByPermissions(route.children, userPermissions, adminLevel)
        : undefined
    }))
    .filter(route => !route.children || route.children.length > 0);
}

export function getNavigationItems(
  userPermissions: Permission[],
  adminLevel?: AdminLevel
): NavigationItem[] {
  const filteredRoutes = filterRoutesByPermissions(ROUTES, userPermissions, adminLevel);
  
  return filteredRoutes
    .filter(route => route.path !== '/' && !route.path.includes('login') && !route.path.includes('register') && !route.path.includes('unauthorized'))
    .map(route => convertRouteToNavItem(route));
}

function convertRouteToNavItem(route: RouteConfig): NavigationItem {
  return {
    title: route.title,
    url: route.path,
    icon: route.icon,
    permissions: route.permissions,
    admin_levels: route.admin_levels,
    children: route.children?.map(convertRouteToNavItem)
  };
}

// ===== ROUTE CHECKING FUNCTIONS =====
export function isPublicRoute(path: string): boolean {
  const publicPaths = [
    '/',
    '/login',
    '/register',
    '/unauthorized'
  ];
  
  return publicPaths.includes(path) || 
         path.startsWith('/api') ||
         path.startsWith('/_app') ||
         path.startsWith('/favicon');
}

export function requiresAuthentication(path: string): boolean {
  return !isPublicRoute(path);
}

export function getRequiredPermissions(path: string): Permission[] {
  const route = findRouteByPath(path);
  return route?.permissions || [];
}

export function getRequiredAdminLevels(path: string): AdminLevel[] {
  const route = findRouteByPath(path);
  return route?.admin_levels || [];
}

function findRouteByPath(path: string): RouteConfig | undefined {
  for (const route of ROUTES) {
    if (route.path === path) {
      return route;
    }
    
    if (route.children) {
      const childRoute = route.children.find(child => child.path === path);
      if (childRoute) {
        return childRoute;
      }
    }
  }
  
  return undefined;
}

// ===== BREADCRUMB HELPERS =====
export function getBreadcrumbs(path: string): NavigationItem[] {
  const breadcrumbs: NavigationItem[] = [];
  const pathSegments = path.split('/').filter(Boolean);
  
  let currentPath = '';
  for (const segment of pathSegments) {
    currentPath += '/' + segment;
    const route = findRouteByPath(currentPath);
    
    if (route) {
      breadcrumbs.push({
        title: route.title,
        url: route.path,
        icon: route.icon,
        permissions: route.permissions,
        admin_levels: route.admin_levels
      });
    }
  }
  
  return breadcrumbs;
}

// ===== PERMISSION-BASED ROUTING =====
export class PermissionRouter {
  constructor(
    private userPermissions: Permission[],
    private adminLevel?: AdminLevel
  ) {}

  canAccess(path: string): boolean {
    const route = findRouteByPath(path);
    if (!route) return false;
    
    return hasRoutePermission(route, this.userPermissions, this.adminLevel);
  }

  getDefaultRoute(): string {
    if (this.adminLevel === 'SuperAdmin') {
      return '/admin/system/dashboard';
    } else if (this.adminLevel === 'FacultyAdmin') {
      return '/admin/faculty/dashboard';
    } else if (this.adminLevel === 'RegularAdmin') {
      return '/admin/scanner';
    } else {
      return '/dashboard';
    }
  }

  getAvailableRoutes(): RouteConfig[] {
    return filterRoutesByPermissions(ROUTES, this.userPermissions, this.adminLevel);
  }

  getNavigationItems(): NavigationItem[] {
    return getNavigationItems(this.userPermissions, this.adminLevel);
  }
}

// ===== ROLE-BASED MENU CONFIGURATIONS =====
export const ROLE_MENUS = {
  student: [
    { title: 'แดshboard', url: '/dashboard', icon: 'dashboard' },
    { title: 'QR Code ของฉัน', url: '/qr', icon: 'qr_code' },
    { title: 'กิจกรรมของฉัน', url: '/activities', icon: 'event' },
    { title: 'โปรไฟล์', url: '/profile', icon: 'person' }
  ],
  
  regular_admin: [
    { title: 'สแกน QR', url: '/admin/scanner', icon: 'qr_code_scanner' },
    { title: 'กิจกรรมที่รับผิดชอบ', url: '/admin/assigned-activities', icon: 'assignment' },
    { title: 'เซสชันของฉัน', url: '/admin/sessions', icon: 'devices' },
    { title: 'โปรไฟล์', url: '/profile', icon: 'person' }
  ],
  
  faculty_admin: [
    { title: 'แดshboard คณะ', url: '/admin/faculty/dashboard', icon: 'analytics' },
    { title: 'จัดการนักศึกษา', url: '/admin/faculty/students', icon: 'group' },
    { title: 'จัดการกิจกรรม', url: '/admin/faculty/activities', icon: 'event' },
    { title: 'รายงาน', url: '/admin/faculty/reports', icon: 'assessment' },
    { title: 'จัดการเซสชัน', url: '/admin/faculty/sessions', icon: 'security' },
    { title: 'สแกน QR', url: '/admin/scanner', icon: 'qr_code_scanner' },
    { title: 'โปรไฟล์', url: '/profile', icon: 'person' }
  ],
  
  super_admin: [
    { title: 'ภาพรวมระบบ', url: '/admin/system/dashboard', icon: 'analytics' },
    { title: 'จัดการคณะ', url: '/admin/system/faculties', icon: 'account_balance' },
    { title: 'จัดการผู้ใช้', url: '/admin/system/users', icon: 'people' },
    { title: 'จัดการผู้ดูแลระบบ', url: '/admin/system/admins', icon: 'admin_panel_settings' },
    { title: 'จัดการเซสชัน', url: '/admin/system/sessions', icon: 'security' },
    { title: 'ตั้งค่าระบบ', url: '/admin/system/settings', icon: 'settings' },
    { title: 'โปรไฟล์', url: '/profile', icon: 'person' }
  ]
};

// ===== QUICK ACCESS SHORTCUTS =====
export const QUICK_ACTIONS = {
  student: [
    { title: 'ดู QR Code', url: '/qr', icon: 'qr_code' },
    { title: 'กิจกรรมล่าสุด', url: '/activities', icon: 'history' }
  ],
  
  regular_admin: [
    { title: 'สแกน QR', url: '/admin/scanner', icon: 'qr_code_scanner' },
    { title: 'กิจกรรมวันนี้', url: '/admin/assigned-activities?date=today', icon: 'today' }
  ],
  
  faculty_admin: [
    { title: 'สแกน QR', url: '/admin/scanner', icon: 'qr_code_scanner' },
    { title: 'สร้างกิจกรรม', url: '/admin/faculty/activities/create', icon: 'add' },
    { title: 'รายงานวันนี้', url: '/admin/faculty/reports?date=today', icon: 'assessment' }
  ],
  
  super_admin: [
    { title: 'ภาพรวมระบบ', url: '/admin/system/dashboard', icon: 'analytics' },
    { title: 'เพิ่มคณะ', url: '/admin/system/faculties/create', icon: 'add' },
    { title: 'สร้างผู้ดูแล', url: '/admin/system/admins/create', icon: 'person_add' }
  ]
};