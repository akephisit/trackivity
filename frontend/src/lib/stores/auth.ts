import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';
import { goto } from '$app/navigation';
import { apiClient, isApiSuccess, handleApiError } from '$lib/api/client';
import { sseClient } from '$lib/sse/client';
import type { 
  SessionUser, 
  LoginRequest, 
  RegisterRequest, 
  Permission, 
  AdminLevel,
  UserSession
} from '$lib/types';

// ===== AUTH STATE INTERFACE =====
interface AuthState {
  user: SessionUser | null;
  sessions: UserSession[];
  isLoading: boolean;
  isAuthenticated: boolean;
  error: string | null;
}

// ===== INITIAL STATE =====
const initialState: AuthState = {
  user: null,
  sessions: [],
  isLoading: false,
  isAuthenticated: false,
  error: null
};

// ===== AUTH STORE =====
function createAuthStore() {
  const { subscribe, set, update } = writable<AuthState>(initialState);

  return {
    subscribe,
    set,
    update,

    // ===== AUTHENTICATION ACTIONS =====
    async login(credentials: LoginRequest) {
      update(state => ({ ...state, isLoading: true, error: null }));

      try {
        const response = await apiClient.login(credentials);
        
        if (isApiSuccess(response)) {
          const { user, session_id } = response.data;
          
          // Store session for mobile apps
          if (browser && localStorage) {
            localStorage.setItem('session_id', session_id);
          }

          // Update auth state
          update(state => ({
            ...state,
            user,
            isLoading: false,
            isAuthenticated: true,
            error: null
          }));

          // Connect to SSE for real-time updates
          sseClient.connect(user);

          return { success: true, user };
        }
      } catch (error) {
        const errorMessage = handleApiError(error);
        update(state => ({
          ...state,
          user: null,
          isLoading: false,
          isAuthenticated: false,
          error: errorMessage
        }));
        return { success: false, error: errorMessage };
      }

      return { success: false, error: 'Login failed' };
    },

    async register(userData: RegisterRequest) {
      update(state => ({ ...state, isLoading: true, error: null }));

      try {
        const response = await apiClient.register(userData);
        
        if (isApiSuccess(response)) {
          update(state => ({
            ...state,
            isLoading: false,
            error: null
          }));

          return { success: true, user: response.data };
        }
      } catch (error) {
        const errorMessage = handleApiError(error);
        update(state => ({
          ...state,
          isLoading: false,
          error: errorMessage
        }));
        return { success: false, error: errorMessage };
      }

      return { success: false, error: 'Registration failed' };
    },

    async logout(redirectTo = '/login') {
      update(state => ({ ...state, isLoading: true }));

      try {
        await apiClient.logout();
      } catch (error) {
        console.error('Logout error:', error);
      }

      // Clear local state
      update(() => ({ ...initialState }));
      
      // Disconnect SSE
      sseClient.disconnect();

      // Clear mobile session storage
      if (browser && localStorage) {
        localStorage.removeItem('session_id');
      }

      // Redirect to login
      if (browser) {
        goto(redirectTo);
      }
    },

    async refreshUser() {
      try {
        const response = await apiClient.me();
        
        if (isApiSuccess(response)) {
          const user = response.data;
          
          update(state => ({
            ...state,
            user,
            isAuthenticated: true,
            error: null
          }));

          // Connect SSE if not already connected and user is authenticated
          if (!sseClient.isConnected()) {
            console.log('[Auth] Connecting SSE for existing session...');
            sseClient.connect(user);
          }

          return user;
        }
      } catch (error) {
        console.log('[Auth] Refresh user failed:', error);
        
        // Handle specific auth errors
        if (error instanceof Error) {
          const errorMessage = error.message;
          if (errorMessage.includes('SESSION_EXPIRED') || 
              errorMessage.includes('SESSION_REVOKED') || 
              errorMessage.includes('SESSION_INVALID') || 
              errorMessage.includes('NO_SESSION') ||
              errorMessage.includes('Authentication failed')) {
            // Clear auth state for session issues but don't redirect (already handled in API client)
            console.log('[Auth] Clearing auth state due to session error');
            update(state => ({
              ...state,
              user: null,
              isAuthenticated: false,
              error: null
            }));
            sseClient.disconnect();
          } else {
            // Log unexpected errors but don't clear auth state
            console.error('[Auth] Unexpected error during refresh:', error);
          }
        }
      }
      return null;
    },

    async refreshSession() {
      try {
        const response = await apiClient.refreshSession();
        
        if (isApiSuccess(response)) {
          const { user, session_id } = response.data;
          
          if (browser && localStorage) {
            localStorage.setItem('session_id', session_id);
          }

          update(state => ({
            ...state,
            user,
            isAuthenticated: true,
            error: null
          }));

          return user;
        }
      } catch (error) {
        console.error('Session refresh failed:', error);
        this.logout();
      }
      return null;
    },

    // ===== SESSION MANAGEMENT =====
    async loadUserSessions() {
      try {
        const response = await apiClient.getUserSessions();
        
        if (isApiSuccess(response)) {
          update(state => ({
            ...state,
            sessions: response.data
          }));
        }
      } catch (error) {
        console.error('Failed to load user sessions:', error);
      }
    },

    async terminateSession(sessionId: string) {
      try {
        const response = await apiClient.terminateSession(sessionId);
        
        if (isApiSuccess(response)) {
          update(state => ({
            ...state,
            sessions: state.sessions.filter(s => s.session_id !== sessionId)
          }));
          return true;
        }
      } catch (error) {
        console.error('Failed to terminate session:', error);
      }
      return false;
    },

    async terminateAllOtherSessions() {
      try {
        const response = await apiClient.terminateAllSessions();
        
        if (isApiSuccess(response)) {
          // Reload sessions to get current state
          await this.loadUserSessions();
          return true;
        }
      } catch (error) {
        console.error('Failed to terminate sessions:', error);
      }
      return false;
    },

    // ===== STATE MANAGEMENT =====
    setUser(user: SessionUser | null) {
      update(state => ({
        ...state,
        user,
        isAuthenticated: !!user
      }));
    },

    setError(error: string | null) {
      update(state => ({ ...state, error }));
    },

    clearError() {
      update(state => ({ ...state, error: null }));
    },

    setLoading(isLoading: boolean) {
      update(state => ({ ...state, isLoading }));
    }
  };
}

// ===== STORE INSTANCE =====
export const auth = createAuthStore();
export const authStore = auth; // Legacy alias for backward compatibility

// ===== DERIVED STORES =====
export const currentUser = derived(auth, $auth => $auth.user);
export const isAuthenticated = derived(auth, $auth => $auth.isAuthenticated);
export const isLoading = derived(auth, $auth => $auth.isLoading);
export const authError = derived(auth, $auth => $auth.error);

// ===== PERMISSION HELPERS =====
export const permissions = derived(currentUser, $user => $user?.permissions || []);

export const adminLevel = derived(currentUser, $user => $user?.admin_role?.admin_level);

export const hasPermission = derived(
  permissions, 
  $permissions => (permission: Permission) => $permissions.includes(permission)
);

export const hasAnyPermission = derived(
  permissions,
  $permissions => (requiredPermissions: Permission[]) => 
    requiredPermissions.some(p => $permissions.includes(p))
);

export const hasAllPermissions = derived(
  permissions,
  $permissions => (requiredPermissions: Permission[]) =>
    requiredPermissions.every(p => $permissions.includes(p))
);

export const isAdmin = derived(
  currentUser,
  $user => !!$user?.admin_role
);

export const isFacultyAdmin = derived(
  currentUser,
  $user => $user?.admin_role?.admin_level === 'FacultyAdmin' || 
           $user?.admin_role?.admin_level === 'SuperAdmin'
);

export const isSuperAdmin = derived(
  currentUser,
  $user => $user?.admin_role?.admin_level === 'SuperAdmin'
);

export const facultyId = derived(
  currentUser,
  $user => $user?.faculty_id || $user?.admin_role?.faculty_id
);

// ===== INITIALIZATION =====
if (browser) {
  // Check if we have a session before attempting to refresh
  const hasSession = () => {
    const cookieMatch = document.cookie.match(/session_id=([^;]+)/);
    return cookieMatch || localStorage.getItem('session_id');
  };

  // Auto-refresh user on page load only if we have a session
  if (hasSession()) {
    console.log('[Auth] Session found, attempting to refresh user...');
    auth.refreshUser();
  } else {
    console.log('[Auth] No session found, skipping auto-refresh');
  }

  // Listen for SSE session events
  sseClient.on('session_updated', (event) => {
    if (event.data?.user) {
      auth.setUser(event.data.user);
    }
  });

  sseClient.on('session_expired', () => {
    auth.logout('/login?expired=true');
  });

  sseClient.on('permission_changed', () => {
    // Refresh user data to get updated permissions
    auth.refreshUser();
  });

  // Session heartbeat - refresh every 15 minutes
  setInterval(() => {
    const currentState = { isAuthenticated: false };
    auth.subscribe(state => Object.assign(currentState, state))();
    
    if (currentState.isAuthenticated && hasSession()) {
      auth.refreshUser();
    }
  }, 15 * 60 * 1000);
}

// ===== UTILITY FUNCTIONS =====
export function requireAuth(): SessionUser {
  let user: SessionUser | null = null;
  currentUser.subscribe(u => user = u)();
  
  if (!user) {
    throw new Error('Authentication required');
  }
  
  return user;
}

export function requirePermission(permission: Permission): SessionUser {
  const user = requireAuth();
  
  if (!user.permissions.includes(permission)) {
    throw new Error(`Permission required: ${permission}`);
  }
  
  return user;
}

export function requireAdminLevel(level: AdminLevel): SessionUser {
  const user = requireAuth();
  
  if (!user.admin_role) {
    throw new Error('Admin access required');
  }
  
  const levels: AdminLevel[] = ['RegularAdmin', 'FacultyAdmin', 'SuperAdmin'];
  const userLevel = user.admin_role.admin_level;
  const requiredLevel = level;
  
  if (levels.indexOf(userLevel) < levels.indexOf(requiredLevel)) {
    throw new Error(`Admin level required: ${level}`);
  }
  
  return user;
}