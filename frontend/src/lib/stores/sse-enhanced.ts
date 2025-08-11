import { writable, derived, get } from 'svelte/store';
import { browser } from '$app/environment';
import { authStore } from './auth';
import { goto } from '$app/navigation';
import { toast } from 'svelte-sonner';

// Enhanced types for SSE messages (keep existing types and add new ones)
export interface SseMessage {
    event_type: string;
    data: any;
    timestamp: string;
    message_id: string;
    target_permissions?: string[];
    target_user_id?: string;
    target_faculty_id?: string;
    target_sessions?: string[];
    priority: MessagePriority;
    ttl_seconds?: number;
    retry_count: number;
    broadcast_id?: string;
}

export enum MessagePriority {
    Low = 'Low',
    Normal = 'Normal',
    High = 'High',
    Critical = 'Critical'
}

export enum SseEventType {
    ActivityCheckedIn = 'activity_checked_in',
    NewActivityCreated = 'new_activity_created',
    SubscriptionExpiryWarning = 'subscription_expiry_warning',
    SystemAnnouncement = 'system_announcement',
    AdminAssignment = 'admin_assignment',
    PermissionUpdated = 'permission_updated',
    SessionRevoked = 'session_revoked',
    AdminPromoted = 'admin_promoted',
    Heartbeat = 'heartbeat',
    ConnectionStatus = 'connection_status',
    Custom = 'custom'
}

// Enhanced notification interface
export interface NotificationMessage {
    title: string;
    message: string;
    notification_type: 'Info' | 'Warning' | 'Error' | 'Success';
    action_url?: string;
    expires_at?: string;
    timestamp?: string;
    read_receipt_required?: boolean;
    sound_enabled?: boolean;
    category?: 'Activity' | 'System' | 'Admin' | 'Security' | 'Subscription';
}

// New specific message types
export interface ActivityCheckedInMessage {
    activity_id: string;
    activity_title: string;
    user_id: string;
    user_name: string;
    checked_in_at: string;
    qr_code_id?: string;
}

export interface NewActivityMessage {
    activity_id: string;
    title: string;
    description?: string;
    start_time: string;
    end_time: string;
    faculty_id?: string;
    created_by: string;
}

export interface SystemAnnouncementMessage {
    announcement_id: string;
    title: string;
    content: string;
    severity: 'Info' | 'Important' | 'Critical' | 'Maintenance';
    target_audience: string[];
    display_until?: string;
}

export interface PermissionUpdatedMessage {
    user_id: string;
    updated_permissions: string[];
    updated_by: string;
    effective_immediately: boolean;
    requires_re_login: boolean;
}

export interface SessionRevokedMessage {
    session_id: string;
    user_id: string;
    reason: 'AdminAction' | 'SecurityBreach' | 'PolicyViolation' | 'PermissionChange' | 'Expired' | 'UserRequested';
    message: string;
    revoked_by?: string;
    force_logout_all_devices: boolean;
}

export interface AdminPromotedMessage {
    user_id: string;
    user_name: string;
    old_role?: string;
    new_role: string;
    promoted_by: string;
    effective_date: string;
    congratulations_message: string;
}

export interface ConnectionStatusMessage {
    status: 'Connected' | 'Reconnecting' | 'Disconnected' | 'Error' | 'RateLimited' | 'Unauthorized';
    message: string;
    reconnect_recommended: boolean;
}

export interface HeartbeatMessage {
    server_time: string;
    connection_count: number;
    uptime_seconds: number;
}

// Keep existing interfaces for compatibility
export interface SessionUpdateMessage {
    session_id: string;
    action: string;
    reason?: string;
    new_expires_at?: string;
}

export interface ActivityUpdateMessage {
    activity_id: string;
    title: string;
    update_type: string;
    message: string;
}

// Enhanced SSE store state
interface SseState {
    connected: boolean;
    connecting: boolean;
    error: string | null;
    lastMessage: SseMessage | null;
    notifications: NotificationMessage[];
    connectionCount: number;
    connectionStats?: ConnectionStats;
    lastHeartbeat?: string;
    messagesReceived: number;
    duplicateMessages: Set<string>;
    reconnectAttempts: number;
    connectionStartTime?: string;
    serverTime?: string;
    role?: 'student' | 'admin';
}

export interface ConnectionStats {
    total_connections: number;
    connections_by_faculty: Record<string, number>;
    connections_by_role: Record<string, number>;
    average_connection_duration: number;
    stale_connections: number;
}

const initialState: SseState = {
    connected: false,
    connecting: false,
    error: null,
    lastMessage: null,
    notifications: [],
    connectionCount: 0,
    messagesReceived: 0,
    duplicateMessages: new Set<string>(),
    reconnectAttempts: 0
};

// Create writable store
export const sseStore = writable<SseState>(initialState);

// Derived stores
export const isConnected = derived(sseStore, $sse => $sse.connected);
export const notifications = derived(sseStore, $sse => $sse.notifications);
export const unreadCount = derived(notifications, $notifications => 
    $notifications.filter(n => !n.expires_at || new Date(n.expires_at) > new Date()).length
);

// Enhanced SSE Service class
class EnhancedSseService {
    private eventSource: EventSource | null = null;
    private maxReconnectAttempts = 10;
    private reconnectAttempt = 0;
    private reconnectTimeout: NodeJS.Timeout | null = null;
    private heartbeatTimeout: NodeJS.Timeout | null = null;
    private messageBuffer: Map<string, SseMessage> = new Map();
    private connectionStartTime: Date | null = null;
    // Connection tracking variables
    private lastEventId: string | null = null;

    // Enhanced connect to SSE endpoint
    connect(sessionId: string, role?: 'student' | 'admin'): void {
        if (!browser || this.eventSource?.readyState === EventSource.OPEN) {
            return;
        }

        this.cleanup();
        this.connectionStartTime = new Date();
        
        sseStore.update(state => ({
            ...state,
            connecting: true,
            error: null,
            reconnectAttempts: this.reconnectAttempt,
            connectionStartTime: this.connectionStartTime?.toISOString(),
            role
        }));

        try {
            // Select appropriate endpoint based on role
            let url = `/api/sse/${sessionId}`;
            if (role === 'student') {
                url = `/api/sse/student/${sessionId}`;
            } else if (role === 'admin') {
                url = `/api/sse/admin/${sessionId}`;
            }

            // Add last event ID for resumption
            if (this.lastEventId) {
                url += `?lastEventId=${encodeURIComponent(this.lastEventId)}`;
            }

            this.eventSource = new EventSource(url, {
                withCredentials: true
            });

            this.setupEventListeners();
            this.startHeartbeatMonitor();
            
        } catch (error) {
            console.error('Failed to create SSE connection:', error);
            sseStore.update(state => ({
                ...state,
                connecting: false,
                error: error instanceof Error ? error.message : 'Connection failed',
                reconnectAttempts: this.reconnectAttempt
            }));
        }
    }

    private startHeartbeatMonitor(): void {
        this.clearHeartbeatTimeout();
        
        // Expect heartbeat every 30 seconds, timeout after 90 seconds
        this.heartbeatTimeout = setTimeout(() => {
            console.warn('SSE heartbeat timeout - connection may be stale');
            sseStore.update(state => ({
                ...state,
                error: 'Connection heartbeat timeout'
            }));
            
            this.handleReconnect();
        }, 90000);
    }

    private clearHeartbeatTimeout(): void {
        if (this.heartbeatTimeout) {
            clearTimeout(this.heartbeatTimeout);
            this.heartbeatTimeout = null;
        }
    }

    private setupEventListeners(): void {
        if (!this.eventSource) return;

        this.eventSource.onopen = () => {
            console.log('SSE connection established');
            this.reconnectAttempt = 0;
            
            sseStore.update(state => ({
                ...state,
                connected: true,
                connecting: false,
                error: null,
                reconnectAttempts: 0
            }));

            // Show success toast for reconnection
            if (this.reconnectAttempt > 0) {
                toast.success('เชื่อมต่อแบบเรียลไทม์สำเร็จแล้ว', {
                    description: 'กลับมาออนไลน์แล้ว'
                });
            }
        };

        this.eventSource.onerror = (event) => {
            console.error('SSE connection error:', event);
            this.clearHeartbeatTimeout();
            
            sseStore.update(state => ({
                ...state,
                connected: false,
                connecting: false,
                error: 'Connection error occurred',
                reconnectAttempts: this.reconnectAttempt
            }));

            // Show error toast only on first attempt
            if (this.reconnectAttempt === 0) {
                toast.error('การเชื่อมต่อขาดหาย', {
                    description: 'กำลังพยายามเชื่อมต่อใหม่...'
                });
            }

            this.handleReconnect();
        };

        this.eventSource.onmessage = (event) => {
            this.handleGenericMessage(event);
        };

        // Enhanced event listeners for all message types
        Object.values(SseEventType).forEach(eventType => {
            this.eventSource?.addEventListener(eventType, (event) => {
                this.handleSpecificEvent(eventType, event as MessageEvent);
            });
        });
    }

    private handleGenericMessage(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            
            // Store last event ID for resumption
            if (event.lastEventId) {
                this.lastEventId = event.lastEventId;
            }

            // Check for duplicate messages
            if (message.message_id && this.isDuplicateMessage(message.message_id)) {
                console.debug('Ignoring duplicate SSE message:', message.message_id);
                return;
            }

            this.processMessage(message);
        } catch (error) {
            console.error('Failed to parse SSE message:', error);
        }
    }

    private handleSpecificEvent(eventType: SseEventType, event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            
            // Store last event ID
            if (event.lastEventId) {
                this.lastEventId = event.lastEventId;
            }

            // Process based on event type
            switch (eventType) {
                case SseEventType.Heartbeat:
                    this.handleHeartbeat(message);
                    break;
                case SseEventType.ConnectionStatus:
                    this.handleConnectionStatus(message);
                    break;
                case SseEventType.ActivityCheckedIn:
                    this.handleActivityCheckedIn(message);
                    break;
                case SseEventType.NewActivityCreated:
                    this.handleNewActivityCreated(message);
                    break;
                case SseEventType.SystemAnnouncement:
                    this.handleSystemAnnouncement(message);
                    break;
                case SseEventType.PermissionUpdated:
                    this.handlePermissionUpdated(message);
                    break;
                case SseEventType.SessionRevoked:
                    this.handleSessionRevoked(message);
                    break;
                case SseEventType.AdminPromoted:
                    this.handleAdminPromoted(message);
                    break;
                default:
                    this.processMessage(message);
                    break;
            }
        } catch (error) {
            console.error(`Failed to handle ${eventType} event:`, error);
        }
    }

    private processMessage(message: SseMessage): void {
        // Add to message buffer for deduplication
        if (message.message_id) {
            this.messageBuffer.set(message.message_id, message);
            
            // Clean old messages (keep last 100)
            if (this.messageBuffer.size > 100) {
                const keys = Array.from(this.messageBuffer.keys());
                this.messageBuffer.delete(keys[0]);
            }
        }

        sseStore.update(state => ({
            ...state,
            lastMessage: message,
            messagesReceived: state.messagesReceived + 1,
            duplicateMessages: message.message_id 
                ? new Set([...state.duplicateMessages, message.message_id])
                : state.duplicateMessages
        }));

        console.log('SSE message processed:', message.event_type, message);
    }

    private isDuplicateMessage(messageId: string): boolean {
        return this.messageBuffer.has(messageId);
    }

    private handleHeartbeat(message: SseMessage): void {
        const heartbeatData = message.data as HeartbeatMessage;
        
        sseStore.update(state => ({
            ...state,
            lastHeartbeat: heartbeatData.server_time,
            connectionCount: heartbeatData.connection_count,
            serverTime: heartbeatData.server_time
        }));

        // Reset heartbeat timeout
        this.startHeartbeatMonitor();
        
        console.debug('Heartbeat received:', heartbeatData);
    }

    private handleConnectionStatus(message: SseMessage): void {
        const statusData = message.data as ConnectionStatusMessage;
        
        sseStore.update(state => ({
            ...state,
            error: statusData.status === 'Connected' ? null : statusData.message
        }));

        // Show appropriate toast based on status
        switch (statusData.status) {
            case 'Connected':
                toast.success('เชื่อมต่อสำเร็จ', {
                    description: statusData.message
                });
                break;
            case 'RateLimited':
                toast.warning('การเชื่อมต่อถูกจำกัด', {
                    description: 'กรุณารอสักครู่แล้วลองใหม่'
                });
                break;
            case 'Error':
                toast.error('เกิดข้อผิดพลาด', {
                    description: statusData.message
                });
                break;
        }
    }

    private handleActivityCheckedIn(message: SseMessage): void {
        const checkinData = message.data as ActivityCheckedInMessage;
        
        const notification: NotificationMessage = {
            title: 'มีผู้เข้าร่วมกิจกรรม',
            message: `${checkinData.user_name} เข้าร่วม "${checkinData.activity_title}"`,
            notification_type: 'Info',
            action_url: `/activities/${checkinData.activity_id}`,
            expires_at: new Date(Date.now() + 3600000).toISOString(), // 1 hour
            category: 'Activity',
            sound_enabled: true
        };

        this.addNotification(notification);
        this.showBrowserNotification(notification);
    }

    private handleNewActivityCreated(message: SseMessage): void {
        const activityData = message.data as NewActivityMessage;
        
        const notification: NotificationMessage = {
            title: 'กิจกรรมใหม่',
            message: `กิจกรรมใหม่: "${activityData.title}"`,
            notification_type: 'Info',
            action_url: `/activities/${activityData.activity_id}`,
            expires_at: new Date(Date.now() + 86400000).toISOString(), // 24 hours
            category: 'Activity',
            sound_enabled: true
        };

        this.addNotification(notification);
        this.showBrowserNotification(notification);
        
        toast.info('กิจกรรมใหม่', {
            description: `"${activityData.title}" พร้อมให้เข้าร่วมแล้ว`
        });
    }

    private handleSystemAnnouncement(message: SseMessage): void {
        const announcement = message.data as SystemAnnouncementMessage;
        
        const notification: NotificationMessage = {
            title: announcement.title,
            message: announcement.content,
            notification_type: announcement.severity === 'Critical' ? 'Error' : 
                             announcement.severity === 'Important' ? 'Warning' : 'Info',
            expires_at: announcement.display_until,
            category: 'System',
            sound_enabled: announcement.severity !== 'Info'
        };

        this.addNotification(notification);
        this.showBrowserNotification(notification);

        // Show appropriate toast
        const toastType = announcement.severity === 'Critical' ? 'error' :
                         announcement.severity === 'Important' ? 'warning' : 'info';
        
        toast[toastType](announcement.title, {
            description: announcement.content,
            duration: announcement.severity === 'Critical' ? 10000 : 7000
        });

        if (announcement.severity !== 'Info') {
            this.playNotificationSound(toastType);
        }
    }

    private handlePermissionUpdated(message: SseMessage): void {
        const permissionData = message.data as PermissionUpdatedMessage;
        
        const notification: NotificationMessage = {
            title: 'สิทธิ์การเข้าถึงได้รับการอัปเดต',
            message: 'สิทธิ์การเข้าถึงของคุณได้รับการเปลี่ยนแปลง',
            notification_type: 'Info',
            category: 'Security',
            sound_enabled: true
        };

        this.addNotification(notification);
        
        toast.info('สิทธิ์อัปเดต', {
            description: permissionData.requires_re_login ? 
                'กรุณาเข้าสู่ระบบใหม่' : 'สิทธิ์ของคุณได้รับการอัปเดต'
        });

        // Force refresh user data
        this.refreshUserData();
    }

    private handleSessionRevoked(message: SseMessage): void {
        const revocationData = message.data as SessionRevokedMessage;
        
        const notification: NotificationMessage = {
            title: 'เซสชันถูกยกเลิก',
            message: revocationData.message,
            notification_type: 'Error',
            category: 'Security',
            sound_enabled: true
        };

        this.addNotification(notification);
        
        // Disconnect and redirect to login
        this.disconnect();
        
        toast.error('เซสชันถูกยกเลิก', {
            description: revocationData.message,
            duration: 10000
        });

        // Clear auth and redirect
        authStore.set({
            user: null,
            session_id: null,
            expires_at: null,
            loading: false,
            error: 'Session revoked by administrator'
        });

        if (browser) {
            localStorage.removeItem('session_id');
            localStorage.removeItem('user');
            localStorage.removeItem('expires_at');
        }

        setTimeout(() => {
            goto('/login?message=session_revoked');
        }, 3000);
    }

    private handleAdminPromoted(message: SseMessage): void {
        const promotionData = message.data as AdminPromotedMessage;
        
        const notification: NotificationMessage = {
            title: 'ยินดีด้วย! คุณได้รับการเลื่อนตำแหน่ง',
            message: promotionData.congratulations_message,
            notification_type: 'Success',
            category: 'Admin',
            sound_enabled: true
        };

        this.addNotification(notification);
        this.showBrowserNotification(notification);
        
        toast.success('ยินดีด้วย!', {
            description: `คุณได้รับการเลื่อนเป็น ${promotionData.new_role}`,
            duration: 10000
        });

        this.playNotificationSound('success');
        
        // Refresh user data to get new permissions
        this.refreshUserData();
    }

    private addNotification(notification: NotificationMessage): void {
        sseStore.update(state => ({
            ...state,
            notifications: [notification, ...state.notifications].slice(0, 50) // Keep last 50
        }));
    }

    private showBrowserNotification(notification: NotificationMessage): void {
        if (!browser || Notification.permission !== 'granted') {
            return;
        }

        new Notification(notification.title, {
            body: notification.message,
            icon: '/favicon.ico',
            tag: 'trackivity-notification',
            requireInteraction: notification.notification_type === 'Error'
        });
    }

    private playNotificationSound(type: string): void {
        if (!browser) return;

        try {
            const audio = new Audio();
            
            switch (type) {
                case 'error':
                    audio.src = '/sounds/error.mp3';
                    break;
                case 'warning':
                    audio.src = '/sounds/warning.mp3';
                    break;
                case 'success':
                    audio.src = '/sounds/success.mp3';
                    break;
                default:
                    audio.src = '/sounds/notification.mp3';
            }

            audio.volume = 0.3;
            audio.play().catch(() => {
                // Ignore autoplay errors
            });
        } catch (error) {
            // Ignore audio errors
        }
    }

    private async refreshUserData(): Promise<void> {
        try {
            const response = await fetch('/api/auth/me', {
                credentials: 'include'
            });

            if (response.ok) {
                const user = await response.json();
                authStore.update(state => ({
                    ...state,
                    user
                }));

                if (browser) {
                    localStorage.setItem('user', JSON.stringify(user));
                }
            }
        } catch (error) {
            console.error('Failed to refresh user data:', error);
        }
    }

    private handleReconnect(): void {
        if (this.reconnectAttempt >= this.maxReconnectAttempts) {
            console.error('Max reconnection attempts reached');
            sseStore.update(state => ({
                ...state,
                error: 'ไม่สามารถเชื่อมต่อได้ กรุณาโหลดหน้าเว็บใหม่'
            }));
            
            toast.error('การเชื่อมต่อล้มเหลว', {
                description: 'กรุณาโหลดหน้าเว็บใหม่',
                duration: 10000
            });
            
            return;
        }

        this.reconnectAttempt++;
        const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempt - 1), 30000);

        console.log(`Attempting to reconnect in ${delay}ms (attempt ${this.reconnectAttempt})`);

        sseStore.update(state => ({
            ...state,
            reconnectAttempts: this.reconnectAttempt
        }));

        this.reconnectTimeout = setTimeout(() => {
            const auth = get(authStore);
            if (auth.session_id) {
                const currentState = get(sseStore);
                this.connect(auth.session_id, currentState.role);
            }
        }, delay);
    }

    disconnect(): void {
        if (this.reconnectTimeout) {
            clearTimeout(this.reconnectTimeout);
            this.reconnectTimeout = null;
        }

        this.clearHeartbeatTimeout();
        this.cleanup();

        sseStore.update(state => ({
            ...state,
            connected: false,
            connecting: false,
            reconnectAttempts: 0
        }));
    }

    private cleanup(): void {
        if (this.eventSource) {
            this.eventSource.close();
            this.eventSource = null;
        }
    }

    // Public methods for notification management
    clearNotifications(): void {
        sseStore.update(state => ({
            ...state,
            notifications: []
        }));
    }

    removeNotification(index: number): void {
        sseStore.update(state => ({
            ...state,
            notifications: state.notifications.filter((_, i) => i !== index)
        }));
    }

    markAsRead(index: number): void {
        sseStore.update(state => {
            const notification = state.notifications[index];
            if (notification && notification.expires_at && new Date(notification.expires_at) <= new Date()) {
                return {
                    ...state,
                    notifications: state.notifications.filter((_, i) => i !== index)
                };
            }
            return state;
        });
    }

    async requestNotificationPermission(): Promise<boolean> {
        if (!browser || !('Notification' in window)) {
            return false;
        }

        if (Notification.permission === 'granted') {
            return true;
        }

        if (Notification.permission === 'denied') {
            return false;
        }

        const permission = await Notification.requestPermission();
        return permission === 'granted';
    }

    // API methods for testing
    async sendTestNotification(title: string, message: string, type: 'Info' | 'Warning' | 'Error' | 'Success' = 'Info'): Promise<boolean> {
        try {
            const response = await fetch('/api/sse/admin/test-notification', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                credentials: 'include',
                body: JSON.stringify({
                    title,
                    message,
                    type: type.toLowerCase()
                })
            });

            return response.ok;
        } catch (error) {
            console.error('Failed to send test notification:', error);
            return false;
        }
    }

    // Get connection statistics for admin users
    async getConnectionStats(): Promise<ConnectionStats | null> {
        try {
            const response = await fetch('/api/sse/admin/stats', {
                credentials: 'include'
            });

            if (response.ok) {
                const stats = await response.json();
                sseStore.update(state => ({
                    ...state,
                    connectionStats: stats
                }));
                return stats;
            }
        } catch (error) {
            console.error('Failed to get connection stats:', error);
        }
        return null;
    }

    // Force cleanup connections (admin only)
    async forceCleanup(): Promise<boolean> {
        try {
            const response = await fetch('/api/sse/admin/cleanup', {
                method: 'POST',
                credentials: 'include'
            });

            if (response.ok) {
                const result = await response.json();
                toast.success('การทำความสะอาดสำเร็จ', {
                    description: `ล้าง ${result.cleaned_connections} การเชื่อมต่อ`
                });
                return true;
            }
        } catch (error) {
            console.error('Failed to force cleanup:', error);
        }
        return false;
    }
}

// Create singleton instance
export const enhancedSseService = new EnhancedSseService();

// Auto-connect when authenticated with role detection
if (browser) {
    authStore.subscribe(auth => {
        if (auth.session_id && !get(sseStore).connected) {
            // Determine role from user data
            const role = auth.user?.admin_role ? 'admin' : 'student';
            enhancedSseService.connect(auth.session_id, role);
        } else if (!auth.session_id && get(sseStore).connected) {
            enhancedSseService.disconnect();
        }
    });

    // Request notification permission on first load
    enhancedSseService.requestNotificationPermission();
}

// Cleanup on page unload
if (browser) {
    window.addEventListener('beforeunload', () => {
        enhancedSseService.disconnect();
    });
}

// Export the original service for compatibility
export { enhancedSseService as sseService };