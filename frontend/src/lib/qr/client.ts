/**
 * QR Code Client for Generation and Scanning
 * รองรับ offline generation และ Web Crypto API signature
 */

import { browser } from '$app/environment';
import { writable, type Writable, get } from 'svelte/store';
import { apiClient } from '$lib/api/client';
import type { QRCode, QRScanResult, SessionUser } from '$lib/types';

// ===== QR CODE CONFIGURATION =====
interface QRConfig {
  size: number;
  margin: number;
  color: string;
  backgroundColor: string;
  errorCorrectionLevel: 'L' | 'M' | 'Q' | 'H';
  refreshInterval: number; // minutes
}

const DEFAULT_QR_CONFIG: QRConfig = {
  size: 256,
  margin: 4,
  color: '#000000',
  backgroundColor: '#ffffff',
  errorCorrectionLevel: 'M',
  refreshInterval: 3 // 3 minutes
};

// ===== QR CODE STATUS =====
export type QRStatus = 'idle' | 'generating' | 'ready' | 'expired' | 'error';

// ===== QR CODE DATA STRUCTURE =====
interface QRData {
  user_id: string;
  timestamp: number;
  session_id: string;
  device_fingerprint: string;
  signature?: string;
}

// ===== DEVICE FINGERPRINTING =====
function generateDeviceFingerprint(): string {
  if (!browser) return 'server-side';

  const components = [
    navigator.userAgent,
    navigator.language,
    screen.width + 'x' + screen.height,
    screen.colorDepth,
    new Date().getTimezoneOffset(),
    !!window.sessionStorage,
    !!window.localStorage,
    !!window.indexedDB,
    typeof (window as any).openDatabase,
    (navigator as any).cpuClass,
    navigator.userAgent, // Replace deprecated platform
    navigator.doNotTrack
  ];

  return btoa(components.join('|')).slice(0, 32);
}

// ===== CRYPTO UTILITIES =====
class CryptoHelper {
  private static async importKey(keyData: string): Promise<CryptoKey> {
    const keyBuffer = new TextEncoder().encode(keyData);
    return await crypto.subtle.importKey(
      'raw',
      keyBuffer,
      { name: 'HMAC', hash: 'SHA-256' },
      false,
      ['sign', 'verify']
    );
  }

  static async signData(data: string, sessionId: string): Promise<string> {
    if (!browser || !crypto.subtle) {
      throw new Error('Web Crypto API not available');
    }

    try {
      const key = await this.importKey(sessionId);
      const dataBuffer = new TextEncoder().encode(data);
      const signature = await crypto.subtle.sign('HMAC', key, dataBuffer);
      return btoa(String.fromCharCode(...new Uint8Array(signature)));
    } catch (error) {
      console.error('Failed to sign QR data:', error);
      throw new Error('Failed to generate signature');
    }
  }

  static async verifySignature(data: string, signature: string, sessionId: string): Promise<boolean> {
    if (!browser || !crypto.subtle) return false;

    try {
      const key = await this.importKey(sessionId);
      const dataBuffer = new TextEncoder().encode(data);
      const signatureBuffer = Uint8Array.from(atob(signature), c => c.charCodeAt(0));
      return await crypto.subtle.verify('HMAC', key, signatureBuffer, dataBuffer);
    } catch (error) {
      console.error('Failed to verify signature:', error);
      return false;
    }
  }
}

// ===== QR CODE GENERATOR =====
export class QRGenerator {
  private canvas: HTMLCanvasElement | null = null;
  private ctx: CanvasRenderingContext2D | null = null;
  private config: QRConfig;

  constructor(config: Partial<QRConfig> = {}) {
    this.config = { ...DEFAULT_QR_CONFIG, ...config };
    
    if (browser) {
      this.setupCanvas();
    }
  }

  private setupCanvas(): void {
    this.canvas = document.createElement('canvas');
    this.canvas.width = this.config.size;
    this.canvas.height = this.config.size;
    this.ctx = this.canvas.getContext('2d');
  }

  // Simple QR Code generation (basic implementation)
  // For production, consider using a library like 'qrcode' or 'qrious'
  generateQRCodeDataURL(data: string): string {
    if (!this.canvas || !this.ctx) {
      throw new Error('Canvas not initialized');
    }

    // Clear canvas
    this.ctx.fillStyle = this.config.backgroundColor;
    this.ctx.fillRect(0, 0, this.config.size, this.config.size);

    // This is a simplified QR code generation
    // In production, use a proper QR code library
    const qrData = this.generateQRMatrix(data);
    this.drawQRMatrix(qrData);

    return this.canvas.toDataURL('image/png');
  }

  private generateQRMatrix(data: string): boolean[][] {
    // Simplified QR matrix generation
    // This is a placeholder - use a proper QR library in production
    const size = 21; // Standard QR code size for version 1
    const matrix: boolean[][] = [];
    
    for (let i = 0; i < size; i++) {
      matrix[i] = [];
      for (let j = 0; j < size; j++) {
        // Generate pseudo-random pattern based on data
        const hash = this.simpleHash(data + i + j);
        matrix[i][j] = hash % 2 === 0;
      }
    }

    return matrix;
  }

  private simpleHash(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash);
  }

  private drawQRMatrix(matrix: boolean[][]): void {
    if (!this.ctx) return;

    const moduleSize = (this.config.size - 2 * this.config.margin) / matrix.length;
    
    this.ctx.fillStyle = this.config.color;
    
    for (let i = 0; i < matrix.length; i++) {
      for (let j = 0; j < matrix[i].length; j++) {
        if (matrix[i][j]) {
          this.ctx.fillRect(
            this.config.margin + j * moduleSize,
            this.config.margin + i * moduleSize,
            moduleSize,
            moduleSize
          );
        }
      }
    }
  }
}

// ===== QR CODE CLIENT =====
export class QRClient {
  private generator: QRGenerator;
  private refreshTimer: NodeJS.Timeout | null = null;
  private config: QRConfig;
  private isGenerating = false;
  private lastGeneratedAt = 0;

  // Svelte stores
  public qrCode: Writable<QRCode | null> = writable(null);
  public qrDataURL: Writable<string | null> = writable(null);
  public status: Writable<QRStatus> = writable('idle');
  public error: Writable<string | null> = writable(null);

  constructor(config: Partial<QRConfig> = {}) {
    this.config = { ...DEFAULT_QR_CONFIG, ...config };
    this.generator = new QRGenerator(config);

    if (browser) {
      // Auto-refresh on page visibility
      document.addEventListener('visibilitychange', () => {
        if (!document.hidden && this.shouldRefreshQR()) {
          this.generateQRCode();
        }
      });

      // Listen for QR refresh events from SSE
      window.addEventListener('qr-refresh', () => {
        this.generateQRCode();
      });
    }
  }

  // ===== QR CODE GENERATION =====
  async generateQRCode(user?: SessionUser): Promise<void> {
    if (!browser) return;
    const now = Date.now();
    // Short-circuit if a generation is in-flight or fired too recently
    if (this.isGenerating || now - this.lastGeneratedAt < 300) {
      return;
    }

    this.isGenerating = true;
    this.status.set('generating');
    this.error.set(null);

    try {
      // Try to generate via API first
      let qrCode: QRCode;
      try {
        const response = await apiClient.generateQRCode();
        // The backend may return either a wrapped shape { status, data } or raw QR payload
        const payload: any = (response as any)?.data ?? (response as any);
        if (!payload || !payload.qr_data) {
          throw new Error('Invalid QR response');
        }

        // Normalize expires_at into ISO string
        let expiresAt: string = payload.expires_at;
        if (typeof payload.expires_at === 'number') {
          expiresAt = new Date(payload.expires_at * 1000).toISOString();
        }

        // Build QRCode object matching app expectations
        qrCode = {
          id: (globalThis.crypto?.randomUUID?.() ?? Math.random().toString(36).slice(2)),
          user_id: user?.user_id || '',
          qr_data: payload.qr_data,
          signature: payload.signature || '',
          created_at: new Date().toISOString(),
          expires_at: expiresAt,
          is_active: true,
          usage_count: 0,
          device_fingerprint: generateDeviceFingerprint()
        };
      } catch (apiError) {
        // Fallback to offline generation
        console.warn('API generation failed, using offline mode:', apiError);
        const sessionId = this.getSessionId();
        qrCode = await this.generateOfflineQRCode(sessionId || undefined, user);
      }

      // Generate visual QR code
      const qrDataURL = this.generator.generateQRCodeDataURL(qrCode.qr_data);

      // Update stores
      this.qrCode.set(qrCode);
      this.qrDataURL.set(qrDataURL);
      this.status.set('ready');

      // Schedule automatic refresh
      this.scheduleRefresh();

    } catch (error) {
      console.error('QR generation failed:', error);
      this.error.set(error instanceof Error ? error.message : 'QR generation failed');
      this.status.set('error');
    } finally {
      this.isGenerating = false;
      this.lastGeneratedAt = Date.now();
    }
  }

  private async generateOfflineQRCode(sessionId?: string, user?: SessionUser): Promise<QRCode> {
    const qrData: QRData = {
      user_id: user?.user_id || 'unknown',
      timestamp: Date.now(),
      session_id: sessionId || 'unknown',
      device_fingerprint: generateDeviceFingerprint()
    };

    // Sign the data if possible
    try {
      if (sessionId) {
        const dataString = JSON.stringify({
          user_id: qrData.user_id,
          timestamp: qrData.timestamp,
          device_fingerprint: qrData.device_fingerprint
        });
        
        qrData.signature = await CryptoHelper.signData(dataString, sessionId);
      }
    } catch (error) {
      console.warn('Failed to sign QR data:', error);
    }

    const expiresAt = new Date(Date.now() + this.config.refreshInterval * 60 * 1000).toISOString();

    return {
      id: crypto.randomUUID(),
      user_id: qrData.user_id,
      qr_data: JSON.stringify(qrData),
      signature: qrData.signature || '',
      created_at: new Date().toISOString(),
      expires_at: expiresAt,
      is_active: true,
      usage_count: 0,
      device_fingerprint: qrData.device_fingerprint
    };
  }

  // ===== QR CODE SCANNING =====
  async scanQRCode(qrData: string, activityId?: string): Promise<QRScanResult> {
    if (!browser) {
      throw new Error('QR scanning not available on server');
    }

    try {
      // Validate QR data format
      const parsedData = this.parseQRData(qrData);
      if (!parsedData) {
        throw new Error('Invalid QR code format');
      }

      // Verify signature if present
      if (parsedData.signature) {
        const isValid = await this.verifyQRSignature(parsedData);
        if (!isValid) {
          throw new Error('Invalid QR code signature');
        }
      }

      // Send to backend for processing
      const response = await apiClient.scanQRCode(qrData, activityId);
      return response.data!;

    } catch (error) {
      console.error('QR scanning failed:', error);
      return {
        success: false,
        scan_time: new Date().toISOString(),
        error: error instanceof Error ? error.message : 'QR scanning failed'
      };
    }
  }

  private parseQRData(qrData: string): QRData | null {
    try {
      const parsed = JSON.parse(qrData);
      if (parsed.user_id && parsed.timestamp && parsed.session_id) {
        return parsed as QRData;
      }
    } catch (error) {
      // Not JSON format, might be simple string
    }
    return null;
  }

  private async verifyQRSignature(qrData: QRData): Promise<boolean> {
    if (!qrData.signature) return false;

    try {
      const dataString = JSON.stringify({
        user_id: qrData.user_id,
        timestamp: qrData.timestamp,
        device_fingerprint: qrData.device_fingerprint
      });

      return await CryptoHelper.verifySignature(
        dataString,
        qrData.signature,
        qrData.session_id
      );
    } catch (error) {
      console.error('Signature verification failed:', error);
      return false;
    }
  }

  // ===== REFRESH MANAGEMENT =====
  private scheduleRefresh(): void {
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
    }

    const refreshMs = this.config.refreshInterval * 60 * 1000;
    this.refreshTimer = setTimeout(() => {
      this.generateQRCode();
    }, refreshMs);
  }

  private shouldRefreshQR(): boolean {
    const currentQR = get(this.qrCode);
    
    if (!currentQR) return true;

    const expiresAt = new Date(currentQR.expires_at).getTime();
    const now = Date.now();
    const timeUntilExpiry = expiresAt - now;
    
    // Refresh if expiring in next minute
    return timeUntilExpiry < 60000;
  }

  // ===== UTILITY METHODS =====
  private getSessionId(): string | null { return null; }

  public downloadQRCode(filename = 'qr-code.png'): void {
    const dataURL = get(this.qrDataURL);
    
    if (!dataURL || !browser) return;

    const link = document.createElement('a');
    link.download = filename;
    link.href = dataURL;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }

  public destroy(): void {
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = null;
    }
  }
}

// ===== SINGLETON INSTANCE =====
export const qrClient = new QRClient();

// ===== CAMERA QR SCANNER =====
export class QRScanner {
  private video: HTMLVideoElement | null = null;
  private canvas: HTMLCanvasElement | null = null;
  private ctx: CanvasRenderingContext2D | null = null;
  private stream: MediaStream | null = null;
  private scanning = false;

  public scanning$: Writable<boolean> = writable(false);
  public error$: Writable<string | null> = writable(null);

  async startCamera(): Promise<void> {
    if (!browser || !navigator.mediaDevices) {
      throw new Error('Camera access not available');
    }

    try {
      this.stream = await navigator.mediaDevices.getUserMedia({
        video: { 
          width: { ideal: 640 },
          height: { ideal: 480 },
          facingMode: 'environment' // Use back camera on mobile
        }
      });

      if (this.video) {
        this.video.srcObject = this.stream;
        await this.video.play();
        this.scanning = true;
        this.scanning$.set(true);
        this.error$.set(null);
      }
    } catch (error) {
      console.error('Failed to start camera:', error);
      this.error$.set('Failed to access camera');
      throw error;
    }
  }

  stopCamera(): void {
    if (this.stream) {
      this.stream.getTracks().forEach(track => track.stop());
      this.stream = null;
    }

    if (this.video) {
      this.video.srcObject = null;
    }

    this.scanning = false;
    this.scanning$.set(false);
  }

  setupVideo(videoElement: HTMLVideoElement, canvasElement: HTMLCanvasElement): void {
    this.video = videoElement;
    this.canvas = canvasElement;
    this.ctx = this.canvas.getContext('2d');
  }

  // This is a placeholder for QR detection
  // In production, use a library like 'qr-scanner' or 'jsqr'
  scanFrame(): string | null {
    if (!this.video || !this.canvas || !this.ctx || !this.scanning) {
      return null;
    }

    // Copy video frame to canvas
    this.canvas.width = this.video.videoWidth;
    this.canvas.height = this.video.videoHeight;
    this.ctx.drawImage(this.video, 0, 0);

    // Get image data for QR detection (placeholder)
    // const imageData = this.ctx.getImageData(0, 0, this.canvas.width, this.canvas.height);
    
    // TODO: Implement QR code detection algorithm
    // This is a placeholder that returns null
    // Use a library like jsQR for actual QR detection
    
    return null;
  }
}

// ===== UTILITY FUNCTIONS =====
export function createQRClient(config?: Partial<QRConfig>): QRClient {
  return new QRClient(config);
}

export function createQRScanner(): QRScanner {
  return new QRScanner();
}

// ===== COMPOSABLE FOR SVELTE COMPONENTS =====
export function useQRCode() {
  const qrCode = qrClient.qrCode;
  const qrDataURL = qrClient.qrDataURL;
  const status = qrClient.status;
  const error = qrClient.error;

  return {
    qrCode,
    qrDataURL,
    status,
    error,
    generate: () => qrClient.generateQRCode(),
    download: (filename?: string) => qrClient.downloadQRCode(filename),
    scan: (qrData: string, activityId?: string) => qrClient.scanQRCode(qrData, activityId)
  };
}
