import { writable, derived, type Readable } from 'svelte/store';
import { browser } from '$app/environment';
import { goto } from '$app/navigation';
import type { User as AdminUser, AdminRole, AuthSession } from '$lib/types/admin';
import type { LoginCredentials as AdminLoginCredentials, RegisterData as AdminRegisterData } from '$lib/types/admin';

// Current User interface for the app (extends AdminUser with additional fields)
export interface User extends AdminUser {
    user_id?: string;
    student_id?: string;
    first_name?: string;
    last_name?: string;
    department_id?: string;
    admin_role?: AdminRole;
    permissions?: string[];
    faculty_id?: string;
    session_id?: string;
}

// Legacy interfaces for backward compatibility
export interface LegacyUser {
    user_id: string;
    student_id: string;
    email: string;
    first_name: string;
    last_name: string;
    department_id?: string;
    admin_role?: AdminRole;
    permissions: string[];
    faculty_id?: string;
    session_id: string;
}

export interface LoginCredentials extends AdminLoginCredentials {
    remember_me?: boolean;
    device_info?: Record<string, any>;
}

export interface RegisterData extends AdminRegisterData {
    student_id?: string;
    first_name?: string;
    last_name?: string;
    department_id?: string;
}

export interface SessionInfo {
    session_id: string;
    device_info: Record<string, any>;
    ip_address?: string;
    user_agent?: string;
    created_at: string;
    last_accessed: string;
    expires_at: string;
}

// Auth store state
interface AuthState {
    user: User | null;
    session_id: string | null;
    expires_at: string | null;
    loading: boolean;
    error: string | null;
}

// Initial state
const initialState: AuthState = {
    user: null,
    session_id: null,
    expires_at: null,
    loading: false,
    error: null
};

// Create writable store
export const authStore = writable<AuthState>(initialState);

// Derived stores for convenience
export const user: Readable<User | null> = derived(authStore, $auth => $auth.user);
export const isAuthenticated: Readable<boolean> = derived(user, $user => $user !== null);
export const isLoading: Readable<boolean> = derived(authStore, $auth => $auth.loading);
export const authError: Readable<string | null> = derived(authStore, $auth => $auth.error);

// Permission checking
export const hasPermission = derived(user, $user => {
    return (permission: string): boolean => {
        return $user?.permissions?.includes(permission) ?? false;
    };
});

export const isAdmin = derived(user, $user => {
    return $user?.admin_role !== undefined;
});

export const adminLevel = derived(user, $user => {
    return $user?.admin_role?.admin_level ?? null;
});

export const isSuperAdmin = derived(adminLevel, $level => {
    return $level === 'SuperAdmin';
});

export const isFacultyAdmin = derived(adminLevel, $level => {
    return $level === 'SuperAdmin' || $level === 'FacultyAdmin';
});

// Auth service class
class AuthService {
    private baseUrl = '/api/auth';

    async login(credentials: LoginCredentials): Promise<{ success: boolean; message: string }> {
        authStore.update(state => ({ ...state, loading: true, error: null }));

        try {
            // Add device information
            const deviceInfo = credentials.device_info || this.getDeviceInfo();
            
            const response = await fetch(`${this.baseUrl}/login`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                credentials: 'include', // Include cookies
                body: JSON.stringify({
                    ...credentials,
                    device_info: deviceInfo
                })
            });

            const data = await response.json();

            if (response.ok && data.success && data.session) {
                authStore.update(state => ({
                    ...state,
                    user: data.session.user,
                    session_id: data.session.session_id,
                    expires_at: data.session.expires_at,
                    loading: false,
                    error: null
                }));

                // Store session info in localStorage for persistence
                if (browser) {
                    localStorage.setItem('session_id', data.session.session_id);
                    localStorage.setItem('user', JSON.stringify(data.session.user));
                    localStorage.setItem('expires_at', data.session.expires_at);
                }

                return { success: true, message: data.message };
            } else {
                authStore.update(state => ({
                    ...state,
                    loading: false,
                    error: data.message || 'Login failed'
                }));
                return { success: false, message: data.message || 'Login failed' };
            }
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Network error';
            authStore.update(state => ({
                ...state,
                loading: false,
                error: errorMessage
            }));
            return { success: false, message: errorMessage };
        }
    }

    async register(userData: RegisterData): Promise<{ success: boolean; message: string }> {
        authStore.update(state => ({ ...state, loading: true, error: null }));

        try {
            const response = await fetch(`${this.baseUrl}/register`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(userData)
            });

            const data = await response.json();

            if (response.ok && data.success) {
                authStore.update(state => ({
                    ...state,
                    loading: false,
                    error: null
                }));
                return { success: true, message: data.message };
            } else {
                authStore.update(state => ({
                    ...state,
                    loading: false,
                    error: data.message || 'Registration failed'
                }));
                return { success: false, message: data.message || 'Registration failed' };
            }
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Network error';
            authStore.update(state => ({
                ...state,
                loading: false,
                error: errorMessage
            }));
            return { success: false, message: errorMessage };
        }
    }

    async logout(): Promise<void> {
        authStore.update(state => ({ ...state, loading: true }));

        try {
            await fetch(`${this.baseUrl}/logout`, {
                method: 'POST',
                credentials: 'include'
            });
        } catch (error) {
            console.error('Logout request failed:', error);
        }

        // Clear auth state regardless of request success
        this.clearAuthState();
    }

    async checkAuth(): Promise<boolean> {
        // Check if we have session info in localStorage
        if (browser) {
            const sessionId = localStorage.getItem('session_id');
            const userStr = localStorage.getItem('user');
            const expiresAt = localStorage.getItem('expires_at');

            if (sessionId && userStr && expiresAt) {
                // Check if session is expired
                if (new Date(expiresAt) > new Date()) {
                    try {
                        const user = JSON.parse(userStr);
                        authStore.update(state => ({
                            ...state,
                            user,
                            session_id: sessionId,
                            expires_at: expiresAt,
                            loading: false
                        }));
                        return true;
                    } catch (error) {
                        console.error('Failed to parse stored user data:', error);
                        this.clearAuthState();
                    }
                } else {
                    // Session expired, clear it
                    this.clearAuthState();
                }
            }
        }

        // Verify with server
        try {
            const response = await fetch(`${this.baseUrl}/me`, {
                credentials: 'include'
            });

            if (response.ok) {
                const user = await response.json();
                authStore.update(state => ({
                    ...state,
                    user,
                    loading: false
                }));
                return true;
            } else {
                this.clearAuthState();
                return false;
            }
        } catch (error) {
            console.error('Auth check failed:', error);
            this.clearAuthState();
            return false;
        }
    }

    async getSessions(): Promise<SessionInfo[]> {
        try {
            const response = await fetch('/api/auth/my-sessions', {
                credentials: 'include'
            });

            if (response.ok) {
                const data = await response.json();
                return data.sessions || [];
            }
        } catch (error) {
            console.error('Failed to fetch sessions:', error);
        }
        return [];
    }

    async revokeSession(sessionId: string): Promise<boolean> {
        try {
            const response = await fetch(`/api/auth/my-sessions/${sessionId}`, {
                method: 'DELETE',
                credentials: 'include'
            });

            const data = await response.json();
            return data.success || false;
        } catch (error) {
            console.error('Failed to revoke session:', error);
            return false;
        }
    }

    async extendSession(hours: number = 24): Promise<boolean> {
        try {
            const response = await fetch('/api/auth/extend-session', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                credentials: 'include',
                body: JSON.stringify({ hours })
            });

            const data = await response.json();
            
            if (data.success && data.expires_at) {
                authStore.update(state => ({
                    ...state,
                    expires_at: data.expires_at
                }));
                
                if (browser) {
                    localStorage.setItem('expires_at', data.expires_at);
                }
                
                return true;
            }
            
            return false;
        } catch (error) {
            console.error('Failed to extend session:', error);
            return false;
        }
    }

    private clearAuthState(): void {
        authStore.set(initialState);
        
        if (browser) {
            localStorage.removeItem('session_id');
            localStorage.removeItem('user');
            localStorage.removeItem('expires_at');
        }
    }

    private getDeviceInfo(): Record<string, any> {
        if (!browser) return {};

        return {
            device_type: this.getDeviceType(),
            screen_resolution: `${screen.width}x${screen.height}`,
            timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
            language: navigator.language,
            platform: navigator.platform,
            user_agent: navigator.userAgent
        };
    }

    private getDeviceType(): string {
        if (!browser) return 'unknown';

        const userAgent = navigator.userAgent.toLowerCase();
        
        if (/tablet|ipad|playbook|silk/i.test(userAgent)) {
            return 'tablet';
        } else if (/mobile|android|iphone|ipod|blackberry|opera|mini|windows\sce|palm|smartphone|iemobile/i.test(userAgent)) {
            return 'mobile';
        } else {
            return 'desktop';
        }
    }

    // Session monitoring
    startSessionMonitoring(): void {
        if (!browser) return;

        // Check session expiry every minute
        setInterval(() => {
            const expiresAt = localStorage.getItem('expires_at');
            if (expiresAt && new Date(expiresAt) <= new Date()) {
                console.log('Session expired, logging out...');
                this.logout();
                goto('/login');
            }
        }, 60000);

        // Extend session on user activity
        let lastActivity = Date.now();
        const activityEvents = ['mousedown', 'mousemove', 'keypress', 'scroll', 'touchstart'];
        
        const updateActivity = () => {
            lastActivity = Date.now();
        };

        activityEvents.forEach(event => {
            document.addEventListener(event, updateActivity, { passive: true });
        });

        // Auto-extend session if user is active
        setInterval(async () => {
            const timeSinceActivity = Date.now() - lastActivity;
            const expiresAt = localStorage.getItem('expires_at');
            
            if (expiresAt && timeSinceActivity < 300000) { // Active within 5 minutes
                const timeToExpiry = new Date(expiresAt).getTime() - Date.now();
                
                // Extend if expiring within 30 minutes
                if (timeToExpiry < 1800000 && timeToExpiry > 0) {
                    console.log('Auto-extending session due to user activity');
                    await this.extendSession();
                }
            }
        }, 300000); // Check every 5 minutes
    }
}

// Create singleton instance
export const authService = new AuthService();

// Auto-check authentication on page load
if (browser) {
    authService.checkAuth();
    authService.startSessionMonitoring();
}

// Utility functions for components
export function requireAuth() {
    let isAuth = false;
    isAuthenticated.subscribe(value => {
        isAuth = value;
    })();
    
    if (!isAuth) {
        goto('/login');
        return false;
    }
    return true;
}

export function requirePermission(permission: string) {
    let userHasPermission = false;
    hasPermission.subscribe(checkFn => {
        userHasPermission = checkFn(permission);
    })();
    
    if (!userHasPermission) {
        goto('/unauthorized');
        return false;
    }
    return true;
}

export function requireAdmin() {
    let userIsAdmin = false;
    isAdmin.subscribe(value => {
        userIsAdmin = value;
    })();
    
    if (!userIsAdmin) {
        goto('/unauthorized');
        return false;
    }
    return true;
}