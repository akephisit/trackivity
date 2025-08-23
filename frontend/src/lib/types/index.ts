/**
 * Core TypeScript types for Trackivity system
 * เข้ากันได้กับ Rust backend structures
 */

// ===== BASE TYPES =====
export type UUID = string;
export type Timestamp = string; // ISO 8601 format

// ===== USER & SESSION TYPES =====
export interface User {
  user_id: UUID;
  student_id: string;
  email: string;
  first_name: string;
  last_name: string;
  department_id?: UUID;
  faculty_id?: UUID;
  created_at: Timestamp;
  updated_at: Timestamp;
  is_active: boolean;
}

export interface AdminRole {
  id: UUID;
  admin_level: AdminLevel;
  faculty_id?: UUID;
  permissions: Permission[];
  created_at: Timestamp;
  updated_at: Timestamp;
}

export type AdminLevel = 'SuperAdmin' | 'FacultyAdmin' | 'RegularAdmin';

export type Permission = 
  // System-wide permissions
  | 'ViewAllUsers'
  | 'CreateUsers'
  | 'UpdateUsers'
  | 'DeleteUsers'
  | 'ViewAllFaculties'
  | 'CreateFaculties'
  | 'UpdateFaculties'
  | 'DeleteFaculties'
  | 'ViewAllSessions'
  | 'ManageAllSessions'
  | 'ViewSystemAnalytics'
  // Faculty-scoped permissions
  | 'ViewFacultyUsers'
  | 'CreateFacultyUsers'
  | 'UpdateFacultyUsers'
  | 'ViewFacultyAnalytics'
  | 'ManageFacultyActivities'
  | 'ViewFacultySessions'
  | 'ManageFacultySessions'
  // Regular admin permissions
  | 'ViewAssignedActivities'
  | 'ScanQRCodes'
  | 'ViewPersonalSessions'
  // Student permissions
  | 'ViewPersonalQR'
  | 'ViewPersonalHistory';

export interface SessionUser {
  user_id: UUID;
  student_id: string;
  email: string;
  first_name: string;
  last_name: string;
  phone?: string;
  address?: string;
  avatar_url?: string;
  department_id?: UUID;
  faculty_id?: UUID;
  faculty_name?: string;
  department_name?: string;
  admin_role?: AdminRole;
  session_id: string;
  permissions: Permission[];
  expires_at: Timestamp;
  created_at?: Timestamp;
  updated_at?: Timestamp;
}

export interface UserSession {
  session_id: string;
  user_id: UUID;
  created_at: Timestamp;
  expires_at: Timestamp;
  last_activity: Timestamp;
  ip_address?: string;
  user_agent?: string;
  is_active: boolean;
  device_info?: DeviceInfo;
}

export interface DeviceInfo {
  device_type: 'web' | 'mobile' | 'tablet';
  os?: string;
  browser?: string;
  app_version?: string;
}

// ===== FACULTY & DEPARTMENT TYPES =====
export interface Faculty {
  id: UUID;
  name: string;
  code: string;
  description?: string;
  is_active: boolean;
  created_at: Timestamp;
  updated_at: Timestamp;
  departments?: Department[];
  total_students?: number;
  total_activities?: number;
}

export interface Department {
  id: UUID;
  name: string;
  code: string;
  faculty_id: UUID;
  is_active: boolean;
  created_at: Timestamp;
  updated_at: Timestamp;
  faculty?: Faculty;
  total_students?: number;
}

// ===== ACTIVITY TYPES =====
export interface Activity {
  id: UUID;
  name: string;
  description?: string;
  activity_type: ActivityType;
  location?: string;
  faculty_id?: UUID;
  department_id?: UUID;
  start_date: Timestamp;
  end_date: Timestamp;
  max_participants?: number;
  current_participants: number;
  is_active: boolean;
  created_by: UUID;
  created_at: Timestamp;
  updated_at: Timestamp;
  qr_settings?: QRSettings;
  faculty?: Faculty;
  department?: Department;
  creator?: User;
}

export type ActivityType = 'lecture' | 'workshop' | 'seminar' | 'exam' | 'meeting' | 'event';

export interface QRSettings {
  refresh_interval: number; // minutes
  expiry_duration: number; // minutes
  signature_required: boolean;
  location_required: boolean;
  device_limit?: number;
}

export interface ActivityParticipation {
  id: UUID;
  activity_id: UUID;
  user_id: UUID;
  participated_at: Timestamp;
  qr_scan_location?: string;
  device_info?: DeviceInfo;
  verified_by?: UUID;
  activity?: Activity;
  user?: User;
  verifier?: User;
}

// ===== QR CODE TYPES =====
export interface QRCode {
  id: UUID;
  user_id: UUID;
  qr_data: string;
  signature: string;
  created_at: Timestamp;
  expires_at: Timestamp;
  is_active: boolean;
  usage_count: number;
  max_usage?: number;
  device_fingerprint?: string;
}

export interface QRScanResult {
  success: boolean;
  activity_id?: UUID;
  user_id?: UUID;
  scan_time: Timestamp;
  location?: string;
  error?: string;
  participation_id?: UUID;
}

// ===== API RESPONSE TYPES =====
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: ApiError;
  meta?: ResponseMeta;
}

export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, any>;
  field_errors?: Record<string, string[]>;
}

export interface ResponseMeta {
  page?: number;
  per_page?: number;
  total?: number;
  total_pages?: number;
  has_next?: boolean;
  has_prev?: boolean;
}

export interface PaginatedResponse<T> extends ApiResponse<T[]> {
  meta: ResponseMeta;
}

// ===== AUTHENTICATION TYPES =====
export interface LoginRequest {
  email: string;
  password: string;
  remember_me?: boolean;
  device_info?: Partial<DeviceInfo>;
}

export interface LoginResponse {
  user: SessionUser;
  session_id: string;
  expires_at: Timestamp;
  permissions: Permission[];
}

export interface RegisterRequest {
  email: string;
  password: string;
  first_name: string;
  last_name: string;
  student_id: string;
  department_id?: UUID;
}

// ===== SSE & REAL-TIME TYPES =====
export interface SSEEvent<T = any> {
  event_type: SSEEventType;
  data: T;
  timestamp: Timestamp;
  user_id?: UUID;
  session_id?: string;
}

export type SSEEventType = 
  | 'session_updated'
  | 'session_expired'
  | 'permission_changed'
  | 'activity_created'
  | 'activity_updated'
  | 'activity_deleted'
  | 'qr_refresh'
  | 'participation_recorded'
  | 'notification'
  | 'system_alert';

export interface SSEConfig {
  auto_reconnect: boolean;
  reconnect_interval: number; // milliseconds
  max_reconnect_attempts: number;
  heartbeat_interval: number; // milliseconds
}

// ===== ANALYTICS & REPORTING TYPES =====
export interface Analytics {
  total_users: number;
  total_activities: number;
  total_participations: number;
  active_sessions: number;
  faculty_breakdown: FacultyStats[];
  activity_breakdown: ActivityStats[];
  participation_trends: ParticipationTrend[];
}

export interface FacultyStats {
  faculty_id: UUID;
  faculty_name: string;
  total_students: number;
  total_activities: number;
  total_participations: number;
  average_participation_rate: number;
}

export interface ActivityStats {
  activity_type: ActivityType;
  total_count: number;
  total_participants: number;
  average_participants: number;
}

export interface ParticipationTrend {
  date: string; // YYYY-MM-DD
  total_participations: number;
  unique_participants: number;
}

// ===== NOTIFICATION TYPES =====
export interface Notification {
  id: UUID;
  user_id: UUID;
  title: string;
  message: string;
  type: NotificationType;
  priority: NotificationPriority;
  read: boolean;
  created_at: Timestamp;
  expires_at?: Timestamp;
  action_url?: string;
  data?: Record<string, any>;
}

export type NotificationType = 
  | 'info'
  | 'success'
  | 'warning' 
  | 'error'
  | 'session'
  | 'activity'
  | 'system';

export type NotificationPriority = 'low' | 'normal' | 'high' | 'critical';

// ===== FORM & UI STATE TYPES =====
export interface FormState<T = any> {
  data: T;
  errors: Record<string, string[]>;
  loading: boolean;
  success: boolean;
}

export interface TableState {
  page: number;
  per_page: number;
  sort_by: string;
  sort_direction: 'asc' | 'desc';
  filters: Record<string, any>;
  search: string;
}

export interface FilterOption {
  value: string;
  label: string;
  count?: number;
}

// ===== MOBILE & PWA TYPES =====
export interface PWAConfig {
  offline_mode: boolean;
  background_sync: boolean;
  push_notifications: boolean;
  auto_update: boolean;
}

export interface OfflineData {
  user_sessions: UserSession[];
  qr_codes: QRCode[];
  cached_at: Timestamp;
  expires_at: Timestamp;
}

// ===== UTILITY TYPES =====
export type Optional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;
export type RequiredFields<T, K extends keyof T> = T & Required<Pick<T, K>>;

// ===== ROUTE & NAVIGATION TYPES =====
export interface RouteConfig {
  path: string;
  title: string;
  permissions: Permission[];
  admin_levels: AdminLevel[];
  icon?: string;
  children?: RouteConfig[];
}

export interface NavigationItem {
  title: string;
  url: string;
  icon?: string;
  badge?: string | number;
  active?: boolean;
  children?: NavigationItem[];
  permissions?: Permission[];
  admin_levels?: AdminLevel[];
}

// ===== THEME & UI TYPES =====
export type Theme = 'light' | 'dark' | 'system';

export interface UIPreferences {
  theme: Theme;
  sidebar_collapsed: boolean;
  dense_mode: boolean;
  animations_enabled: boolean;
  sound_notifications: boolean;
}

// Re-export commonly used types
export type { User as AppUser };
export type { SessionUser as CurrentUser };
export type { AdminRole as UserRole };