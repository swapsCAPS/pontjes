const OFFLINE_VERSION = 2;
const CACHE_NAME = `offline-${OFFLINE_VERSION}`;

const offlinePage = new Request('/public/offline.html', { cache: 'reload' })
const reqsToCache = [
  new Request('/public/stylesheets/main-v9.css', { cache: 'reload' }),
  new Request('/public/fonts/Roboto-Bold.ttf', { cache: 'reload' }),
  new Request('/public/fonts/Roboto-Regular.ttf', { cache: 'reload' }),
  new Request('/public/favicon.png', { cache: 'reload' }),
  new Request('/public/favicon-192.png', { cache: 'reload' }),
  new Request('/public/favicon-512.png', { cache: 'reload' }),
  new Request('/public/favicon-128.png', { cache: 'reload' }),
  new Request('/public/favicon-152.png', { cache: 'reload' }),
  new Request('/public/favicon-180.png', { cache: 'reload' }),
  new Request('/public/favicon-167.png', { cache: 'reload' }),
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    (async () => {
      const cache = await caches.open(CACHE_NAME);

      return cache.addAll(reqsToCache)
    })()
  );
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    (async () => {
      if ('navigationPreload' in self.registration) {
        await self.registration.navigationPreload.enable();
      }
    })()
  );

  self.clients.claim();
});

self.addEventListener('fetch', (event) => {
  event.respondWith(
    (async () => {
      try {
        const preloadResponse = await event.preloadResponse;
        if (preloadResponse) return preloadResponse;

        const networkResponse = await fetch(event.request);
        return networkResponse;
      } catch (error) {
        console.log('Fetch failed; returning offline page instead.', error);

        const cache = await caches.open(CACHE_NAME);
        let cacheHit = reqsToCache.find(r => r.url === event.request.url);

        // If we don't find any cached url just load the offline page
        const cachedResponse = await cache.match(cacheHit || offlinePage);
        return cachedResponse;
      }
    })()
  );
});
