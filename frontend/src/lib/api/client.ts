/**
 * Type-safe API client for communicating with Rust backend
 * รองรับ session-based authentication และ mobile app headers
 */

import { browser } from '$app/environment';
import { PUBLIC_API_URL as PUBLIC_API_URL_RAW } from '$env/static/public';
import type {
  ApiResponse,
  PaginatedResponse,
  SessionUser,
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  User,
  Faculty,
  Department,
  Activity,
  QRCode,
  QRScanResult,
  UserSession,
  Analytics,
  Notification,
  DeviceInfo
} from '$lib/types';

// ===== CONFIGURATION =====
// In the browser, use same-origin so Vite proxy (dev) or reverse-proxy (prod) can route /api.
// On the server (SSR), fall back to PUBLIC_API_URL or localhost.
const CLEAN_PUBLIC_API_URL = (PUBLIC_API_URL_RAW || '').trim().replace(/\/$/, '');
const API_BASE_URL = browser ? '' : (CLEAN_PUBLIC_API_URL || process.env.PUBLIC_API_URL || 'http://localhost:3000');

const DEFAULT_TIMEOUT = 30000; // 30 seconds

// ===== ERROR TYPES =====
export class ApiClientError extends Error {
  constructor(
    public code: string,
    message: string,
    public status?: number,
    public details?: Record<string, any>
  ) {
    super(message);
    this.name = 'ApiClientError';
  }
}

export class NetworkError extends ApiClientError {
  constructor(message = 'Network error occurred') {
    super('NETWORK_ERROR', message, 0);
  }
}

export class TimeoutError extends ApiClientError {
  constructor(message = 'Request timed out') {
    super('TIMEOUT_ERROR', message, 408);
  }
}

export class AuthenticationError extends ApiClientError {
  constructor(message = 'Authentication failed') {
    super('AUTH_ERROR', message, 401);
  }
}

export class AuthorizationError extends ApiClientError {
  constructor(message = 'Access denied') {
    super('AUTHORIZATION_ERROR', message, 403);
  }
}

// ===== CLIENT CONFIGURATION =====
export interface ApiClientConfig {
  baseUrl?: string;
  timeout?: number;
  retryAttempts?: number;
  retryDelay?: number;
  defaultHeaders?: Record<string, string>;
}

// ===== REQUEST OPTIONS =====
export interface RequestOptions extends RequestInit {
  timeout?: number;
  retries?: number;
  skipAuth?: boolean;
  customHeaders?: Record<string, string>;
}

// ===== MAIN API CLIENT CLASS =====
export class ApiClient {
  private baseUrl: string;
  private timeout: number;
  private retryAttempts: number;
  private retryDelay: number;
  private defaultHeaders: Record<string, string>;

  constructor(config: ApiClientConfig = {}) {
    this.baseUrl = config.baseUrl || API_BASE_URL;
    this.timeout = config.timeout || DEFAULT_TIMEOUT;
    this.retryAttempts = config.retryAttempts || 3;
    this.retryDelay = config.retryDelay || 1000;
    this.defaultHeaders = {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
      ...config.defaultHeaders
    };
  }

  // ===== PRIVATE HELPERS =====
  // Browser no longer needs to read session; proxy attaches httpOnly cookie.
  // Keeping this method for potential future use or debugging
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  private getSessionId(): string | null { return null; }

  private getDeviceInfo(): DeviceInfo {
    if (!browser) return { device_type: 'web' };

    const userAgent = navigator.userAgent;
    const isMobile = /Android|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(userAgent);
    const isTablet = /iPad/i.test(userAgent);

    return {
      device_type: isTablet ? 'tablet' : isMobile ? 'mobile' : 'web',
      os: this.extractOS(userAgent),
      browser: this.extractBrowser(userAgent),
      app_version: (window as any).APP_VERSION || undefined
    };
  }

  private extractOS(userAgent: string): string {
    if (userAgent.includes('Windows')) return 'Windows';
    if (userAgent.includes('Mac')) return 'macOS';
    if (userAgent.includes('Linux')) return 'Linux';
    if (userAgent.includes('Android')) return 'Android';
    if (userAgent.includes('iOS')) return 'iOS';
    return 'Unknown';
  }

  private extractBrowser(userAgent: string): string {
    if (userAgent.includes('Chrome')) return 'Chrome';
    if (userAgent.includes('Firefox')) return 'Firefox';
    if (userAgent.includes('Safari')) return 'Safari';
    if (userAgent.includes('Edge')) return 'Edge';
    return 'Unknown';
  }

  private buildHeaders(options: RequestOptions = {}): Record<string, string> {
    const headers: Record<string, string> = { ...this.defaultHeaders };

    // Add session authentication if not skipped
    if (!options.skipAuth && browser) {
      // Same-origin requests with credentials; proxy attaches cookies server-side
      // Add device info for mobile apps
      const deviceInfo = this.getDeviceInfo();
      if (deviceInfo.device_type !== 'web') {
        headers['X-Device-Type'] = deviceInfo.device_type;
        headers['X-Device-Info'] = JSON.stringify(deviceInfo);
      }
    }

    // Add custom headers
    if (options.customHeaders) {
      Object.assign(headers, options.customHeaders);
    }

    return headers;
  }

  private async makeRequest<T>(
    endpoint: string,
    options: RequestOptions = {}
  ): Promise<ApiResponse<T>> {
    const url = `${this.baseUrl}${endpoint}`;
    const timeout = options.timeout || this.timeout;
    const retries = options.retries !== undefined ? options.retries : this.retryAttempts;
    let lastError: Error | undefined;

    for (let attempt = 0; attempt <= retries; attempt++) {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), timeout);
      const requestOptions: RequestInit = {
        ...options,
        headers: this.buildHeaders(options),
        signal: controller.signal,
        credentials: 'include'
      };

      try {
        const response = await fetch(url, requestOptions);
        clearTimeout(timeoutId);
        return await this.handleResponse<T>(response);
      } catch (err) {
        clearTimeout(timeoutId);
        const handled = this.handleError(err);
        lastError = handled as Error;
        if (attempt < retries && this.shouldRetry(handled as Error)) {
          await this.delay(this.retryDelay * (attempt + 1));
          continue;
        }
        throw handled;
      }
    }

    // Fallback, should not reach here
    throw lastError ?? new NetworkError('Network request failed');
  }

  private async handleResponse<T>(response: Response): Promise<ApiResponse<T>> {
    const contentType = response.headers.get('content-type');
    
    // Handle no-content success responses
    if (response.status === 204) {
      return { success: true } as ApiResponse<T>;
    }
    
    if (!contentType?.includes('application/json')) {
      // Some backend rejections (e.g., auth middleware) may send empty bodies
      if (response.status === 401) {
        throw new AuthenticationError('Authentication failed');
      }
      if (response.status === 403) {
        throw new AuthorizationError('Access denied');
      }
      throw new ApiClientError(
        'INVALID_RESPONSE',
        'Expected JSON response',
        response.status
      );
    }

    const data = await response.json();

    if (!response.ok) {
      // Handle authentication errors
      if (response.status === 401) {
        throw new AuthenticationError(data.error?.message || 'Authentication failed');
      }

      if (response.status === 403) {
        throw new AuthorizationError(data.error?.message || 'Access denied');
      }

      throw new ApiClientError(
        data.error?.code || 'API_ERROR',
        data.error?.message || 'Request failed',
        response.status,
        data.error?.details
      );
    }

    // Handle different error response formats
    if (data.success === false) {
      // New format with explicit error handling
      const errorCode = data.error?.code;
      if (browser && ['SESSION_EXPIRED', 'SESSION_REVOKED', 'SESSION_INVALID', 'NO_SESSION'].includes(errorCode)) {
        console.log(`[Client] Session error detected: ${errorCode}`);
      }
      
      throw new ApiClientError(
        data.error?.code || 'API_ERROR',
        data.error?.message || 'Request failed',
        response.status,
        data.error?.details
      );
    }

    // Handle backend error format: { status: "error", message: "..." }
    if ('status' in data && data.status === 'error') {
      throw new ApiClientError(
        data.code || 'API_ERROR',
        data.message || 'Request failed',
        response.status,
        data.details
      );
    }

    // Normalize different response formats to ApiResponse shape
    if (typeof data === 'object' && data !== null) {
      // Handle backend format: { status: "success", data: {...} }
      if ('status' in data && data.status === 'success' && 'data' in data) {
        return { success: true, data: data.data } as ApiResponse<T>;
      }
      // Handle legacy format: direct data object (without success flag)
      else if (!('success' in data)) {
        return { success: true, data } as ApiResponse<T>;
      }
    }

    return data as ApiResponse<T>;
  }

  private handleError(error: any): Error {
    if (error.name === 'AbortError') {
      return new TimeoutError();
    }

    if (error instanceof ApiClientError) {
      return error;
    }

    if (browser && typeof navigator !== 'undefined' && !navigator.onLine) {
      return new NetworkError('No internet connection');
    }

    return new NetworkError(error.message || 'Network error occurred');
  }

  private shouldRetry(error: Error): boolean {
    return error instanceof NetworkError || 
           error instanceof TimeoutError ||
           (error instanceof ApiClientError && typeof error.status === 'number' && error.status >= 500);
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  // ===== GENERIC REQUEST METHODS =====
  async get<T>(endpoint: string, options?: RequestOptions): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(endpoint, { ...options, method: 'GET' });
  }

  async post<T>(endpoint: string, data?: any, options?: RequestOptions): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(endpoint, {
      ...options,
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined
    });
  }

  async put<T>(endpoint: string, data?: any, options?: RequestOptions): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(endpoint, {
      ...options,
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined
    });
  }

  async patch<T>(endpoint: string, data?: any, options?: RequestOptions): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(endpoint, {
      ...options,
      method: 'PATCH',
      body: data ? JSON.stringify(data) : undefined
    });
  }

  async delete<T>(endpoint: string, options?: RequestOptions): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(endpoint, { ...options, method: 'DELETE' });
  }

  // ===== AUTHENTICATION ENDPOINTS =====
  async login(credentials: LoginRequest): Promise<ApiResponse<LoginResponse>> {
    return this.post<LoginResponse>('/api/auth/login', {
      ...credentials,
      device_info: this.getDeviceInfo()
    }, { skipAuth: true });
  }

  async register(userData: RegisterRequest): Promise<ApiResponse<User>> {
    return this.post<User>('/api/auth/register', userData, { skipAuth: true });
  }

  async logout(): Promise<ApiResponse<void>> {
    // Use a lightweight, resilient call to avoid UI blocking
    try {
      const controller = new AbortController();
      const t = setTimeout(() => controller.abort(), 5000);
      await fetch('/api/auth/logout', {
        method: 'POST',
        credentials: 'include',
        headers: { 'Accept': 'application/json' },
        keepalive: true,
        signal: controller.signal
      }).catch(() => {});
      clearTimeout(t);
    } catch (_) {}
    // Always return success to the caller; server cleanup is idempotent
    return { success: true } as ApiResponse<void>;
  }

  async me(): Promise<ApiResponse<SessionUser>> {
    try {
      const response = await this.get<SessionUser>('/api/auth/me');
      return response;
    } catch (error) {
      // Handle specific auth errors more gracefully
      if (error instanceof ApiClientError) {
        throw error;
      }
      throw new AuthenticationError('Failed to get current user');
    }
  }

  async refreshSession(): Promise<ApiResponse<LoginResponse>> {
    return this.post<LoginResponse>('/api/auth/refresh');
  }

  // ===== USER MANAGEMENT =====
  async getUsers(params?: {
    page?: number;
    per_page?: number;
    search?: string;
    faculty_id?: string;
    department_id?: string;
  }): Promise<PaginatedResponse<User>> {
    const query = new URLSearchParams();
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined) query.append(key, value.toString());
      });
    }
    
    const endpoint = `/api/users${query.toString() ? `?${query.toString()}` : ''}`;
    return this.get<User[]>(endpoint) as Promise<PaginatedResponse<User>>;
  }

  async getUser(userId: string): Promise<ApiResponse<User>> {
    return this.get<User>(`/api/users/${userId}`);
  }

  async createUser(userData: Partial<User>): Promise<ApiResponse<User>> {
    return this.post<User>('/api/users', userData);
  }

  async updateUser(userId: string, userData: Partial<User>): Promise<ApiResponse<User>> {
    return this.put<User>(`/api/users/${userId}`, userData);
  }

  async deleteUser(userId: string): Promise<ApiResponse<void>> {
    return this.delete<void>(`/api/users/${userId}`);
  }

  // ===== FACULTY & DEPARTMENT =====
  async getFaculties(): Promise<ApiResponse<Faculty[]>> {
    return this.get<Faculty[]>('/api/faculties');
  }

  async getFaculty(facultyId: string): Promise<ApiResponse<Faculty>> {
    return this.get<Faculty>(`/api/faculties/${facultyId}`);
  }

  async getFacultyDepartments(facultyId: string): Promise<ApiResponse<Department[]>> {
    return this.get<Department[]>(`/api/faculties/${facultyId}/departments`);
  }

  async getDepartments(): Promise<ApiResponse<Department[]>> {
    return this.get<Department[]>('/api/departments');
  }

  // ===== ACTIVITIES =====
  async getActivities(params?: {
    page?: number;
    per_page?: number;
    faculty_id?: string;
    activity_type?: string;
    active_only?: boolean;
  }): Promise<PaginatedResponse<Activity>> {
    const query = new URLSearchParams();
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined) query.append(key, value.toString());
      });
    }
    
    const endpoint = `/api/activities${query.toString() ? `?${query.toString()}` : ''}`;
    return this.get<Activity[]>(endpoint) as Promise<PaginatedResponse<Activity>>;
  }

  async getActivity(activityId: string): Promise<ApiResponse<Activity>> {
    return this.get<Activity>(`/api/activities/${activityId}`);
  }

  async createActivity(activityData: Partial<Activity>): Promise<ApiResponse<Activity>> {
    return this.post<Activity>('/api/activities', activityData);
  }

  async updateActivity(activityId: string, activityData: Partial<Activity>): Promise<ApiResponse<Activity>> {
    return this.put<Activity>(`/api/activities/${activityId}`, activityData);
  }

  async deleteActivity(activityId: string): Promise<ApiResponse<void>> {
    return this.delete<void>(`/api/activities/${activityId}`);
  }

  // ===== QR CODE MANAGEMENT =====
  async generateQRCode(): Promise<ApiResponse<QRCode>> {
    return this.post<QRCode>('/api/qr/generate');
  }

  async scanQRCode(qrData: string, activityId?: string): Promise<ApiResponse<QRScanResult>> {
    return this.post<QRScanResult>('/api/qr/scan', {
      qr_data: qrData,
      activity_id: activityId,
      device_info: this.getDeviceInfo()
    });
  }

  async getUserQRCodes(): Promise<ApiResponse<QRCode[]>> {
    return this.get<QRCode[]>('/api/qr/user');
  }

  // ===== SESSION MANAGEMENT =====
  async getUserSessions(): Promise<ApiResponse<UserSession[]>> {
    return this.get<UserSession[]>('/api/sessions/user');
  }

  async getAllSessions(params?: {
    page?: number;
    per_page?: number;
    active_only?: boolean;
  }): Promise<PaginatedResponse<UserSession>> {
    const query = new URLSearchParams();
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined) query.append(key, value.toString());
      });
    }
    
    const endpoint = `/api/sessions${query.toString() ? `?${query.toString()}` : ''}`;
    return this.get<UserSession[]>(endpoint) as Promise<PaginatedResponse<UserSession>>;
  }

  async terminateSession(sessionId: string): Promise<ApiResponse<void>> {
    return this.delete<void>(`/api/sessions/${sessionId}`);
  }

  async terminateAllSessions(): Promise<ApiResponse<void>> {
    return this.delete<void>('/api/sessions/all');
  }

  // ===== ANALYTICS =====
  async getAnalytics(): Promise<ApiResponse<Analytics>> {
    return this.get<Analytics>('/api/analytics');
  }

  async getFacultyAnalytics(facultyId: string): Promise<ApiResponse<Analytics>> {
    return this.get<Analytics>(`/api/analytics/faculties/${facultyId}`);
  }

  // ===== NOTIFICATIONS =====
  async getNotifications(): Promise<ApiResponse<Notification[]>> {
    return this.get<Notification[]>('/api/notifications');
  }

  async markNotificationRead(notificationId: string): Promise<ApiResponse<void>> {
    return this.patch<void>(`/api/notifications/${notificationId}/read`);
  }

  async markAllNotificationsRead(): Promise<ApiResponse<void>> {
    return this.post<void>('/api/notifications/read-all');
  }
}

// ===== SINGLETON INSTANCE =====
export const apiClient = new ApiClient();

// ===== UTILITY FUNCTIONS =====
export function isApiError(error: any): error is ApiClientError {
  return error instanceof ApiClientError;
}

export function handleApiError(error: any): string {
  if (isApiError(error)) {
    return error.message;
  }
  return 'An unexpected error occurred';
}

// ===== RESPONSE HELPERS =====
export function isApiSuccess<T>(response: ApiResponse<T>): response is ApiResponse<T> & { success: true; data: T } {
  return response.success && response.data !== undefined;
}

export function unwrapApiResponse<T>(response: ApiResponse<T>): T {
  if (!isApiSuccess(response)) {
    throw new ApiClientError(
      response.error?.code || 'API_ERROR',
      response.error?.message || 'Request failed',
      undefined,
      response.error?.details
    );
  }
  return response.data;
}
