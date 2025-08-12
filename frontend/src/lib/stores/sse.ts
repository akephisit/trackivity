import { writable, derived, get } from 'svelte/store';
import { browser } from '$app/environment';
import { authStore } from './auth';
import { goto } from '$app/navigation';
import { toast } from 'svelte-sonner';

// Enhanced types for SSE messages
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

export interface ConnectionStatusMessage {
    status: 'Connected' | 'Reconnecting' | 'Disconnected' | 'Error' | 'RateLimited' | 'Unauthorized';
    message: string;
    reconnect_recommended: boolean;
}

export interface SessionUpdateMessage {
    session_id: string;
    action: string; // "force_logout" | "permission_changed" | "extended"
    reason?: string;
    new_expires_at?: string;
}

export interface ActivityUpdateMessage {
    activity_id: string;
    title: string;
    update_type: string; // "created" | "updated" | "cancelled" | "started"
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
class SseService {
    private eventSource: EventSource | null = null;
    private maxReconnectAttempts = 10;
    private reconnectAttempt = 0;
    private reconnectTimeout: NodeJS.Timeout | null = null;
    private heartbeatTimeout: NodeJS.Timeout | null = null;
    private messageBuffer: Map<string, SseMessage> = new Map();
    private connectionStartTime: Date | null = null;
    private connectionId: string | null = null;

    // Enhanced connect to SSE endpoint with improved error handling
    connect(): void {
        if (!browser || this.eventSource?.readyState === EventSource.OPEN) {
            return;
        }

        this.cleanup();
        this.connectionId = `${Date.now()}`;
        this.connectionStartTime = new Date();
        
        sseStore.update(state => ({
            ...state,
            connecting: true,
            error: null,
            reconnectAttempts: this.reconnectAttempt,
            connectionStartTime: this.connectionStartTime?.toISOString()
        }));

        try {
            // Use same-origin SSE proxy; backend reads httpOnly cookie
            let url = `/api/sse`;

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

    // Start heartbeat monitoring
    private startHeartbeatMonitor(): void {
        this.clearHeartbeatTimeout();
        
        // Expect heartbeat every 60 seconds, timeout after 90 seconds
        this.heartbeatTimeout = setTimeout(() => {
            console.warn('SSE heartbeat timeout - connection may be stale');
            sseStore.update(state => ({
                ...state,
                error: 'Connection heartbeat timeout'
            }));
            
            // Attempt reconnection
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

            // Show error toast only if not already reconnecting
            if (this.reconnectAttempt === 0) {
                toast.error('การเชื่อมต่อขาดหาย', {
                    description: 'กำลังพยายามเชื่อมต่อใหม่...'
                });
            }

            // Attempt to reconnect
            this.handleReconnect();
        };

        this.eventSource.onmessage = (event) => {
            this.handleMessage(event);
        };

        // Handle specific event types with enhanced processing
        this.eventSource.addEventListener('notification', (event) => {
            this.handleNotification(event);
        });

        this.eventSource.addEventListener('session_update', (event) => {
            this.handleSessionUpdate(event);
        });

        this.eventSource.addEventListener('activity_update', (event) => {
            this.handleActivityUpdate(event);
        });

        this.eventSource.addEventListener('activity_checked_in', (event) => {
            this.handleActivityCheckedIn(event);
        });

        this.eventSource.addEventListener('new_activity_created', (event) => {
            this.handleNewActivityCreated(event);
        });

        this.eventSource.addEventListener('system_announcement', (event) => {
            this.handleSystemAnnouncement(event);
        });

        this.eventSource.addEventListener('permission_updated', (event) => {
            this.handlePermissionUpdated(event);
        });

        this.eventSource.addEventListener('session_revoked', (event) => {
            this.handleSessionRevoked(event);
        });

        this.eventSource.addEventListener('admin_promoted', (event) => {
            this.handleAdminPromoted(event);
        });

        this.eventSource.addEventListener('heartbeat', (event) => {
            this.handleHeartbeat(event);
        });

        this.eventSource.addEventListener('connection_status', (event) => {
            this.handleConnectionStatus(event);
        });

        this.eventSource.addEventListener('error', (event) => {
            console.error('SSE error event:', event);
        });
    }

    private handleMessage(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            
            // Check for duplicate messages
            if (message.message_id && this.isDuplicateMessage(message.message_id)) {
                console.debug('Ignoring duplicate SSE message:', message.message_id);
                return;
            }

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

            console.log('SSE message received:', message);
            
            // Handle message based on priority
            this.handleMessageByPriority(message);
        } catch (error) {
            console.error('Failed to parse SSE message:', error);
        }
    }

    private isDuplicateMessage(messageId: string): boolean {
        return this.messageBuffer.has(messageId);
    }

    private handleMessageByPriority(message: SseMessage): void {
        switch (message.priority) {
            case MessagePriority.Critical:
                // Show critical messages immediately
                toast.error('ข้อความสำคัญ', {
                    description: 'มีข้อความสำคัญที่ต้องการความสนใจ',
                    duration: 10000,
                });
                this.playNotificationSound('error');
                break;
            
            case MessagePriority.High:
                // Show high priority messages prominently
                toast.warning('ข้อความสำคัญ', {
                    description: 'มีข้อความที่ต้องการความสนใจ',
                    duration: 7000,
                });
                this.playNotificationSound('warning');
                break;
            
            default:
                // Normal and low priority messages don't need immediate attention
                break;
        }
    }

    private handleNotification(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            const notification: NotificationMessage = message.data;

            // Add notification to store
            sseStore.update(state => ({
                ...state,
                notifications: [notification, ...state.notifications].slice(0, 50) // Keep last 50
            }));

            // Show browser notification if permission granted
            this.showBrowserNotification(notification);

            // Play notification sound
            this.playNotificationSound((notification.notification_type as string).toLowerCase());

        } catch (error) {
            console.error('Failed to handle notification:', error);
        }
    }

    private handleSessionUpdate(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            const sessionUpdate: SessionUpdateMessage = message.data;

            console.log('Session update received:', sessionUpdate);

            switch (sessionUpdate.action) {
                case 'force_logout':
                    this.handleForceLogout(sessionUpdate);
                    break;
                case 'permission_changed':
                    this.handlePermissionChange(sessionUpdate);
                    break;
                case 'extended':
                    this.handleSessionExtended(sessionUpdate);
                    break;
            }
        } catch (error) {
            console.error('Failed to handle session update:', error);
        }
    }

    private handleActivityUpdate(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            const activityUpdate: ActivityUpdateMessage = message.data;

            // Create notification for activity update
            const notification: NotificationMessage = {
                title: `Activity ${activityUpdate.update_type}`,
                message: `${activityUpdate.title}: ${activityUpdate.message}`,
                notification_type: activityUpdate.update_type === 'cancelled' ? 'Warning' : 'Info',
                action_url: `/activities/${activityUpdate.activity_id}`
            };

            sseStore.update(state => ({
                ...state,
                notifications: [notification, ...state.notifications].slice(0, 50)
            }));

        } catch (error) {
            console.error('Failed to handle activity update:', error);
        }
    }

    // Added minimal handlers for specific SSE event types to satisfy typing
    private handleActivityCheckedIn(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            const data = message.data as ActivityCheckedInMessage;
            const notification: NotificationMessage = {
                title: 'Check-in สำเร็จ',
                message: `${data.user_name} เช็คอินกิจกรรมแล้ว`,
                notification_type: 'Success',
                action_url: `/activities/${data.activity_id}`
            };
            sseStore.update(state => ({
                ...state,
                notifications: [notification, ...state.notifications].slice(0, 50)
            }));
        } catch (error) {
            console.error('Failed to handle activity_checked_in:', error);
        }
    }

    private handleNewActivityCreated(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            const data = message.data as NewActivityMessage;
            const notification: NotificationMessage = {
                title: 'มีกิจกรรมใหม่',
                message: data.title,
                notification_type: 'Info',
                action_url: `/activities/${data.activity_id}`
            };
            sseStore.update(state => ({
                ...state,
                notifications: [notification, ...state.notifications].slice(0, 50)
            }));
        } catch (error) {
            console.error('Failed to handle new_activity_created:', error);
        }
    }

    private handleSystemAnnouncement(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            const ann = message.data as SystemAnnouncementMessage;
            const notification: NotificationMessage = {
                title: ann.title,
                message: ann.content,
                notification_type: 'Info'
            };
            sseStore.update(state => ({
                ...state,
                notifications: [notification, ...state.notifications].slice(0, 50)
            }));
        } catch (error) {
            console.error('Failed to handle system_announcement:', error);
        }
    }

    private handlePermissionUpdated(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            console.log('Permissions updated event:', message);
            const notification: NotificationMessage = {
                title: 'สิทธิ์การใช้งานเปลี่ยนแปลง',
                message: 'ระบบได้อัปเดตสิทธิ์การใช้งานของคุณ',
                notification_type: 'Info'
            };
            sseStore.update(state => ({
                ...state,
                notifications: [notification, ...state.notifications].slice(0, 50)
            }));
            this.refreshUserData();
        } catch (error) {
            console.error('Failed to handle permission_updated:', error);
        }
    }

    private handleSessionRevoked(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            const reason = message.data?.reason || 'Your session has been revoked.';
            this.handleForceLogout({ action: 'force_logout', reason } as SessionUpdateMessage);
        } catch (error) {
            console.error('Failed to handle session_revoked:', error);
        }
    }

    private handleAdminPromoted(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            const notification: NotificationMessage = {
                title: 'ได้รับสิทธิ์ผู้ดูแลระบบ',
                message: 'บัญชีของคุณได้รับการเลื่อนสิทธิ์การจัดการ',
                notification_type: 'Success'
            };
            sseStore.update(state => ({
                ...state,
                notifications: [notification, ...state.notifications].slice(0, 50)
            }));
            this.refreshUserData();
        } catch (error) {
            console.error('Failed to handle admin_promoted:', error);
        }
    }

    private handleHeartbeat(event: MessageEvent): void {
        try {
            const now = new Date().toISOString();
            sseStore.update(state => ({ ...state, lastHeartbeat: now }));
            this.startHeartbeatMonitor();
        } catch {
            // ignore
        }
    }

    private handleConnectionStatus(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            const status = message.data as ConnectionStatusMessage;
            sseStore.update(state => ({
                ...state,
                error: status.status === 'Error' ? status.message : state.error
            }));
        } catch (error) {
            console.error('Failed to handle connection_status:', error);
        }
    }

    private handleForceLogout(sessionUpdate: SessionUpdateMessage): void {
        const notification: NotificationMessage = {
            title: 'Session Terminated',
            message: sessionUpdate.reason || 'Your session has been terminated by an administrator.',
            notification_type: 'Error'
        };

        // Show notification
        sseStore.update(state => ({
            ...state,
            notifications: [notification, ...state.notifications]
        }));

        // Disconnect SSE
        this.disconnect();

        // Clear auth and redirect to login
        authStore.update(state => ({
            ...state,
            user: null,
            isAuthenticated: false,
            isLoading: false,
            error: 'Session terminated by administrator'
        }));

        // No client-side session storage to clear

        // Redirect to login with message
        goto('/login?message=session_terminated');
    }

    private handlePermissionChange(sessionUpdate: SessionUpdateMessage): void {
        const notification: NotificationMessage = {
            title: 'Permissions Updated',
            message: sessionUpdate.reason || 'Your account permissions have been updated.',
            notification_type: 'Info'
        };

        sseStore.update(state => ({
            ...state,
            notifications: [notification, ...state.notifications]
        }));

        // Refresh user data to get updated permissions
        this.refreshUserData();
    }

    private handleSessionExtended(sessionUpdate: SessionUpdateMessage): void {
        if (sessionUpdate.new_expires_at) {
            const notification: NotificationMessage = {
                title: 'Session Extended',
                message: 'Your session has been extended.',
                notification_type: 'Success'
            };

            sseStore.update(state => ({
                ...state,
                notifications: [notification, ...state.notifications]
            }));
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
                    // No client-side storage persistence
                }
            }
        } catch (error) {
            console.error('Failed to refresh user data:', error);
        }
    }

    private showBrowserNotification(notification: NotificationMessage): void {
        if (!browser || Notification.permission !== 'granted') {
            return;
        }

        new Notification(notification.title, {
            body: notification.message,
            icon: '/favicon.ico',
            tag: 'trackivity-notification'
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

    private handleReconnect(): void {
        if (this.reconnectAttempt >= this.maxReconnectAttempts) {
            console.error('Max reconnection attempts reached');
            sseStore.update(state => ({
                ...state,
                error: 'Failed to reconnect after multiple attempts'
            }));
            return;
        }

        this.reconnectAttempt++;
        const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempt), 30000);

        console.log(`Attempting to reconnect in ${delay}ms (attempt ${this.reconnectAttempt})`);

        this.reconnectTimeout = setTimeout(() => {
            this.connect();
        }, delay);
    }

    disconnect(): void {
        if (this.reconnectTimeout) {
            clearTimeout(this.reconnectTimeout);
            this.reconnectTimeout = null;
        }

        this.cleanup();

        sseStore.update(state => ({
            ...state,
            connected: false,
            connecting: false
        }));
    }

    private cleanup(): void {
        if (this.eventSource) {
            this.eventSource.close();
            this.eventSource = null;
        }
    }

    // Clear all notifications
    clearNotifications(): void {
        sseStore.update(state => ({
            ...state,
            notifications: []
        }));
    }

    // Remove specific notification
    removeNotification(index: number): void {
        sseStore.update(state => ({
            ...state,
            notifications: state.notifications.filter((_, i) => i !== index)
        }));
    }

    // Mark notification as read (remove if expired)
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

    // Request notification permission
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
}

// Create singleton instance
export const sseService = new SseService();

// Auto-connect when authenticated
if (browser) {
    authStore.subscribe(auth => {
        const connected = get(sseStore).connected;
        if (auth.isAuthenticated && !connected) {
            sseService.connect();
        } else if (!auth.isAuthenticated && connected) {
            sseService.disconnect();
        }
    });

    // Request notification permission on first load
    sseService.requestNotificationPermission();
}

// Cleanup on page unload
if (browser) {
    window.addEventListener('beforeunload', () => {
        sseService.disconnect();
    });
}
