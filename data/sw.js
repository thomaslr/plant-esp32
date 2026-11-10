// PlantESP32 Service Worker (v1.0.11)
const CACHE_NAME = 'plant-esp32-v13';

const STATIC_ASSETS = [
    '/',
    '/style.css?v=8',
    '/app.js?v=25',
    '/favicon.svg',
    '/icon-192.png',
    '/icon-512.png',
    '/manifest.json'
];

// Install: Cache critical App Shell assets immediately and activate
self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => {
            return cache.addAll(STATIC_ASSETS);
        }).then(() => self.skipWaiting())
    );
});

// Activate: Immediately purge all old cache versions and claim clients
self.addEventListener('activate', (event) => {
    event.waitUntil(
        caches.keys().then((cacheNames) => {
            return Promise.all(
                cacheNames
                    .filter((name) => name !== CACHE_NAME)
                    .map((name) => caches.delete(name))
            );
        }).then(() => self.clients.claim())
    );
});

// Fetch: Never cache non-GET or /api/; Network-First for navigation & scripts so updates appear immediately
self.addEventListener('fetch', (event) => {
    const { request } = event;
    const url = new URL(request.url);

    // 1. Never intercept non-GET requests (pump triggers, config POSTs, OTA uploads)
    if (request.method !== 'GET') {
        return;
    }

    // 2. Real-time API telemetry & endpoints: bypass service worker cache completely
    if (url.pathname.startsWith('/api/')) {
        return;
    }

    // 3. Network-First for navigation (HTML) and scripts so firmware/UI updates load immediately
    event.respondWith(
        fetch(request)
            .then((networkResponse) => {
                if (networkResponse && networkResponse.status === 200) {
                    const responseClone = networkResponse.clone();
                    caches.open(CACHE_NAME).then((cache) => {
                        cache.put(request, responseClone);
                    });
                }
                return networkResponse;
            })
            .catch(() => {
                // Network unavailable (offline) -> serve from cache
                return caches.match(request).then((cachedResponse) => {
                    if (cachedResponse) return cachedResponse;
                    if (request.mode === 'navigate') {
                        return caches.match('/');
                    }
                    return Promise.reject('Offline and asset not cached');
                });
            })
    );
});
