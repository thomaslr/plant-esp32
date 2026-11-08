// PlantESP32 Service Worker (v1.0.0)
const CACHE_NAME = 'plant-esp32-v2';

const STATIC_ASSETS = [
    '/',
    '/style.css?v=6',
    '/app.js?v=14',
    '/favicon.svg',
    '/icon-192.png',
    '/icon-512.png',
    '/manifest.json'
];

// Install: Cache critical App Shell assets immediately
self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => {
            return cache.addAll(STATIC_ASSETS);
        }).then(() => self.skipWaiting())
    );
});

// Activate: Clean up old cache versions and claim clients
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

// Fetch: Never cache non-GET or /api/ calls; network-first for telemetry
self.addEventListener('fetch', (event) => {
    const { request } = event;
    const url = new URL(request.url);

    // 1. Never cache non-GET requests (pump triggers, config POSTs)
    if (request.method !== 'GET') {
        return;
    }

    // 2. Real-time API telemetry: bypass cache completely
    if (url.pathname.startsWith('/api/')) {
        return;
    }

    // 3. App Shell & assets: Stale-While-Revalidate with network fallback
    event.respondWith(
        caches.match(request).then((cachedResponse) => {
            const fetchPromise = fetch(request).then((networkResponse) => {
                if (networkResponse && networkResponse.status === 200) {
                    const responseToCache = networkResponse.clone();
                    caches.open(CACHE_NAME).then((cache) => {
                        cache.put(request, responseToCache);
                    });
                }
                return networkResponse;
            }).catch(() => {
                // If network fails, cachedResponse will be returned
            });

            return cachedResponse || fetchPromise;
        })
    );
});
