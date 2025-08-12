/**
 * Service Worker for Trackivity PWA
 * รองรับ offline functionality, caching, และ background sync
 */

const CACHE_NAME = 'trackivity-v1.0.0';
const STATIC_CACHE_NAME = 'trackivity-static-v1.0.0';
const DYNAMIC_CACHE_NAME = 'trackivity-dynamic-v1.0.0';

// Files to cache immediately
const STATIC_FILES = [
  '/',
  '/offline',
  '/manifest.json'
  // Add other static assets as needed
];

// API endpoints to cache
const CACHEABLE_APIS = [
  '/api/auth/me',
  '/api/faculties',
  '/api/departments'
];

// Files to exclude from caching
const EXCLUDE_PATTERNS = [
  '/api/auth/login',
  '/api/auth/logout',
  '/api/auth/me',
  '/api/sse',
  '/api/qr/generate',
  '/api/qr/scan'
];

// ===== INSTALL EVENT =====
self.addEventListener('install', (event) => {
  console.log('[SW] Installing...');
  
  event.waitUntil(
    Promise.all([
      // Cache static files
      caches.open(STATIC_CACHE_NAME).then((cache) => {
        console.log('[SW] Caching static files');
        return cache.addAll(STATIC_FILES);
      }),
      
      // Skip waiting to activate immediately
      self.skipWaiting()
    ])
  );
});

// ===== ACTIVATE EVENT =====
self.addEventListener('activate', (event) => {
  console.log('[SW] Activating...');
  
  event.waitUntil(
    Promise.all([
      // Clean up old caches
      caches.keys().then((cacheNames) => {
        return Promise.all(
          cacheNames
            .filter((cacheName) => 
              cacheName !== CACHE_NAME && 
              cacheName !== STATIC_CACHE_NAME && 
              cacheName !== DYNAMIC_CACHE_NAME
            )
            .map((cacheName) => {
              console.log('[SW] Deleting old cache:', cacheName);
              return caches.delete(cacheName);
            })
        );
      }),
      
      // Take control of all clients
      self.clients.claim()
    ])
  );
});

// ===== FETCH EVENT =====
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // Skip non-GET requests
  if (request.method !== 'GET') {
    return;
  }

  // Skip excluded patterns
  if (EXCLUDE_PATTERNS.some(pattern => url.pathname.includes(pattern))) {
    return;
  }

  // Handle different types of requests
  if (url.pathname.startsWith('/api/')) {
    event.respondWith(handleApiRequest(request));
  } else if (url.pathname.startsWith('/_app/') || url.pathname.includes('.')) {
    event.respondWith(handleStaticRequest(request));
  } else {
    event.respondWith(handlePageRequest(request));
  }
});

// ===== REQUEST HANDLERS =====

// Handle API requests with network-first strategy
async function handleApiRequest(request) {
  const url = new URL(request.url);
  
  try {
    // Try network first
    const response = await fetch(request);
    
    // Cache successful responses for cacheable APIs
    if (response.ok && CACHEABLE_APIS.some(api => url.pathname.includes(api))) {
      const cache = await caches.open(DYNAMIC_CACHE_NAME);
      cache.put(request, response.clone());
    }
    
    return response;
  } catch (error) {
    console.log('[SW] Network failed, trying cache for:', url.pathname);
    
    // If network fails, try cache
    const cachedResponse = await caches.match(request);
    if (cachedResponse) {
      return cachedResponse;
    }
    
    // Return offline response for specific endpoints
    if (url.pathname === '/api/auth/me') {
      return new Response(JSON.stringify({ 
        success: false, 
        error: { code: 'OFFLINE', message: 'Offline mode' }
      }), {
        status: 503,
        headers: { 'Content-Type': 'application/json' }
      });
    }
    
    throw error;
  }
}

// Handle static files with cache-first strategy
async function handleStaticRequest(request) {
  const cachedResponse = await caches.match(request);
  
  if (cachedResponse) {
    return cachedResponse;
  }
  
  try {
    const response = await fetch(request);
    
    if (response.ok) {
      const cache = await caches.open(STATIC_CACHE_NAME);
      cache.put(request, response.clone());
    }
    
    return response;
  } catch (error) {
    console.log('[SW] Failed to fetch static resource:', request.url);
    throw error;
  }
}

// Handle page requests with network-first, fallback to offline page
async function handlePageRequest(request) {
  try {
    const response = await fetch(request);
    return response;
  } catch (error) {
    console.log('[SW] Page request failed, returning offline page');
    
    // Return cached page or offline page
    const cachedPage = await caches.match(request);
    if (cachedPage) {
      return cachedPage;
    }
    
    const offlinePage = await caches.match('/offline');
    if (offlinePage) {
      return offlinePage;
    }
    
    // Fallback HTML for offline mode
    return new Response(`
      <!DOCTYPE html>
      <html lang="th">
      <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Trackivity - ออฟไลน์</title>
        <style>
          body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
            color: #333;
            text-align: center;
          }
          .offline-icon {
            font-size: 4rem;
            margin-bottom: 1rem;
          }
          .offline-title {
            font-size: 2rem;
            font-weight: 600;
            margin-bottom: 1rem;
          }
          .offline-message {
            font-size: 1.1rem;
            margin-bottom: 2rem;
            max-width: 500px;
            line-height: 1.5;
          }
          .retry-button {
            background: #007bff;
            color: white;
            border: none;
            padding: 12px 24px;
            border-radius: 6px;
            font-size: 1rem;
            cursor: pointer;
            transition: background-color 0.2s;
          }
          .retry-button:hover {
            background: #0056b3;
          }
        </style>
      </head>
      <body>
        <div class="offline-icon">📱</div>
        <h1 class="offline-title">ออฟไลน์</h1>
        <p class="offline-message">
          ไม่สามารถเชื่อมต่ออินเทอร์เน็ตได้ในขณะนี้<br>
          กรุณาตรวจสอบการเชื่อมต่อแล้วลองใหม่อีกครั้ง
        </p>
        <button class="retry-button" onclick="location.reload()">
          ลองใหม่อีกครั้ง
        </button>
        
        <script>
          // Auto-retry when online
          window.addEventListener('online', () => {
            location.reload();
          });
        </script>
      </body>
      </html>
    `, {
      headers: { 'Content-Type': 'text/html' }
    });
  }
}

// ===== BACKGROUND SYNC =====
self.addEventListener('sync', (event) => {
  console.log('[SW] Background sync:', event.tag);
  
  if (event.tag === 'qr-scan-sync') {
    event.waitUntil(syncQRScans());
  } else if (event.tag === 'session-sync') {
    event.waitUntil(syncSessions());
  }
});

// Sync QR scan data when back online
async function syncQRScans() {
  try {
    const db = await openIndexedDB();
    const transaction = db.transaction(['pendingScans'], 'readonly');
    const store = transaction.objectStore('pendingScans');
    const pendingScans = await getAllFromStore(store);
    
    for (const scan of pendingScans) {
      try {
        const response = await fetch('/api/qr/scan', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(scan.data)
        });
        
        if (response.ok) {
          // Remove from pending scans
          const deleteTransaction = db.transaction(['pendingScans'], 'readwrite');
          const deleteStore = deleteTransaction.objectStore('pendingScans');
          await deleteStore.delete(scan.id);
          
          console.log('[SW] Synced QR scan:', scan.id);
        }
      } catch (error) {
        console.error('[SW] Failed to sync QR scan:', error);
      }
    }
  } catch (error) {
    console.error('[SW] Background sync failed:', error);
  }
}

// Sync session data
async function syncSessions() {
  try {
    // Attempt to refresh session
    const response = await fetch('/api/auth/me');
    if (response.ok) {
      console.log('[SW] Session sync successful');
    }
  } catch (error) {
    console.error('[SW] Session sync failed:', error);
  }
}

// ===== PUSH NOTIFICATIONS =====
self.addEventListener('push', (event) => {
  console.log('[SW] Push received');
  
  const data = event.data ? event.data.json() : {};
  const title = data.title || 'Trackivity';
  const options = {
    body: data.body || 'คุณมีการแจ้งเตือนใหม่',
    tag: data.tag || 'general',
    data: data.data || {},
    actions: [
      {
        action: 'view',
        title: 'ดูรายละเอียด'
      },
      {
        action: 'dismiss',
        title: 'ปิด'
      }
    ],
    vibrate: [200, 100, 200]
  };
  
  event.waitUntil(
    self.registration.showNotification(title, options)
  );
});

// Handle notification clicks
self.addEventListener('notificationclick', (event) => {
  console.log('[SW] Notification clicked:', event.action);
  
  event.notification.close();
  
  if (event.action === 'view') {
    const url = event.notification.data.url || '/';
    event.waitUntil(
      clients.matchAll().then((clientList) => {
        if (clientList.length > 0) {
          return clientList[0].focus().then(() => {
            return clientList[0].navigate(url);
          });
        }
        return clients.openWindow(url);
      })
    );
  }
});

// ===== MESSAGE HANDLING =====
self.addEventListener('message', (event) => {
  console.log('[SW] Message received:', event.data);
  
  if (event.data.type === 'SKIP_WAITING') {
    self.skipWaiting();
  } else if (event.data.type === 'CACHE_QR_SCAN') {
    // Cache QR scan for later sync
    cacheQRScanForSync(event.data.payload);
  }
});

// ===== UTILITY FUNCTIONS =====

// Open IndexedDB for offline storage
function openIndexedDB() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open('TrackivityDB', 1);
    
    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);
    
    request.onupgradeneeded = (event) => {
      const db = event.target.result;
      
      if (!db.objectStoreNames.contains('pendingScans')) {
        db.createObjectStore('pendingScans', { keyPath: 'id', autoIncrement: true });
      }
      
      if (!db.objectStoreNames.contains('offlineData')) {
        db.createObjectStore('offlineData', { keyPath: 'key' });
      }
    };
  });
}

// Get all items from IndexedDB store
function getAllFromStore(store) {
  return new Promise((resolve, reject) => {
    const request = store.getAll();
    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);
  });
}

// Cache QR scan for background sync
async function cacheQRScanForSync(scanData) {
  try {
    const db = await openIndexedDB();
    const transaction = db.transaction(['pendingScans'], 'readwrite');
    const store = transaction.objectStore('pendingScans');
    
    await store.add({
      data: scanData,
      timestamp: Date.now()
    });
    
    console.log('[SW] QR scan cached for sync');
    
    // Register background sync
    if ('serviceWorker' in navigator && 'sync' in window.ServiceWorkerRegistration.prototype) {
      await self.registration.sync.register('qr-scan-sync');
    }
  } catch (error) {
    console.error('[SW] Failed to cache QR scan:', error);
  }
}

console.log('[SW] Service Worker loaded');