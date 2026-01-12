# CORS Proxy Configuration

This document explains the CORS proxy feature used in the Sermon Helper application for Bible API requests.

## Overview

The application fetches Bible verses from two external APIs:
- **V2 API** (`api.nyiregyhazimetodista.hu`) - For UF and RUF translations
- **Legacy API** (`szentiras.eu`) - For RUF, KG, KNB, SZIT, BD, STL translations

These APIs may have CORS (Cross-Origin Resource Sharing) restrictions that prevent direct browser requests.

## How It Works

### Tauri Desktop Mode (`pnpm tauri dev`)

When running as a Tauri desktop application, all API requests are made through Rust's `reqwest` HTTP client. This completely bypasses CORS restrictions because:

1. Requests originate from the Rust backend, not the browser
2. The `src-tauri/src/bible.rs` module handles all HTTP requests
3. No browser security policies apply

**This is the recommended way to run the application.**

### Browser Development Mode (`pnpm dev`)

When running in browser-only mode (Vite dev server), requests are subject to browser CORS policies. To handle this, the application uses a CORS proxy service that:

1. Receives requests from the browser
2. Forwards them to the target API
3. Returns the response with proper CORS headers

```
Browser → CORS Proxy → szentiras.eu → CORS Proxy → Browser
```

## Configuration

### Environment Variables

Create a `.env` file in the project root to customize the CORS proxy behavior:

```bash
# Enable/disable CORS proxy (default: true)
VITE_USE_CORS_PROXY=true

# CORS proxy URL (default: https://corsproxy.io/?)
VITE_CORS_PROXY_URL=https://corsproxy.io/?

# API URLs (optional, defaults shown)
VITE_BIBLE_V2_API_URL=https://api.nyiregyhazimetodista.hu
VITE_BIBLE_LEGACY_API_URL=https://szentiras.eu
```

### Runtime Configuration

You can also update the configuration at runtime:

```typescript
import { bibleApi } from '$lib/utils/bible-api';

// Update proxy settings
bibleApi.updateConfig({
  browserProxyUrl: 'https://your-proxy.com/?url=',
  useCorsProxy: true,
});

// Check current configuration
console.log(bibleApi.getConfig());

// Check if running in Tauri or browser
console.log('Is Tauri:', bibleApi.isTauri());
```

## Available CORS Proxy Services

### 1. corsproxy.io (Default)

- **URL:** `https://corsproxy.io/?`
- **Usage:** Append the encoded target URL
- **Pros:** Free, reliable, no rate limits for reasonable usage
- **Cons:** Third-party service

```
https://corsproxy.io/?https%3A%2F%2Fszentiras.eu%2Fapi%2Fidezet%2FJn3%2C16%2FRUF
```

### 2. AllOrigins

- **URL:** `https://api.allorigins.win/raw?url=`
- **Usage:** Append the encoded target URL
- **Pros:** Free, open source
- **Cons:** May have occasional downtime

```
https://api.allorigins.win/raw?url=https%3A%2F%2Fszentiras.eu%2Fapi%2Fidezet%2FJn3%2C16%2FRUF
```

### 3. CORS Anywhere (Heroku)

- **URL:** `https://cors-anywhere.herokuapp.com/`
- **Usage:** Prepend to the target URL (no encoding needed)
- **Pros:** Well-known, open source
- **Cons:** Requires manual activation at https://cors-anywhere.herokuapp.com/corsdemo

```
https://cors-anywhere.herokuapp.com/https://szentiras.eu/api/idezet/Jn3,16/RUF
```

## Setting Up Your Own Proxy

For production use or if public proxies are unreliable, you can set up your own CORS proxy.

### Option 1: Cloudflare Worker (Free)

Create a new Cloudflare Worker with this code:

```javascript
addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request));
});

async function handleRequest(request) {
  const url = new URL(request.url);
  const targetUrl = url.searchParams.get('url');

  if (!targetUrl) {
    return new Response('Missing url parameter', { status: 400 });
  }

  // Only allow specific domains
  const allowed = ['szentiras.eu', 'api.nyiregyhazimetodista.hu'];
  const targetHost = new URL(targetUrl).hostname;

  if (!allowed.some(domain => targetHost.endsWith(domain))) {
    return new Response('Domain not allowed', { status: 403 });
  }

  const response = await fetch(targetUrl, {
    headers: request.headers,
  });

  const newResponse = new Response(response.body, response);
  newResponse.headers.set('Access-Control-Allow-Origin', '*');
  newResponse.headers.set('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  newResponse.headers.set('Access-Control-Allow-Headers', 'Content-Type');

  return newResponse;
}
```

Then configure:
```bash
VITE_CORS_PROXY_URL=https://your-worker.your-subdomain.workers.dev/?url=
```

### Option 2: Vercel Edge Function (Free)

Create `api/proxy.ts` in a Vercel project:

```typescript
import { NextRequest, NextResponse } from 'next/server';

export const config = { runtime: 'edge' };

export default async function handler(req: NextRequest) {
  const { searchParams } = new URL(req.url);
  const targetUrl = searchParams.get('url');

  if (!targetUrl) {
    return NextResponse.json({ error: 'Missing url' }, { status: 400 });
  }

  const response = await fetch(targetUrl);
  const data = await response.text();

  return new NextResponse(data, {
    headers: {
      'Content-Type': response.headers.get('Content-Type') || 'application/json',
      'Access-Control-Allow-Origin': '*',
    },
  });
}
```

### Option 3: Simple Node.js Proxy

```javascript
const express = require('express');
const cors = require('cors');
const fetch = require('node-fetch');

const app = express();
app.use(cors());

app.get('/proxy', async (req, res) => {
  const targetUrl = req.query.url;

  if (!targetUrl) {
    return res.status(400).json({ error: 'Missing url parameter' });
  }

  try {
    const response = await fetch(targetUrl);
    const data = await response.json();
    res.json(data);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

app.listen(3001, () => console.log('Proxy running on port 3001'));
```

Configure:
```bash
VITE_CORS_PROXY_URL=http://localhost:3001/proxy?url=
```

## Troubleshooting

### CORS errors in browser mode

1. **Check if proxy is enabled:**
   ```typescript
   console.log(bibleApi.getConfig().useCorsProxy);
   ```

2. **Verify proxy URL is correct:**
   ```typescript
   console.log(bibleApi.getConfig().browserProxyUrl);
   ```

3. **Test proxy manually:**
   ```bash
   curl "https://corsproxy.io/?https%3A%2F%2Fszentiras.eu%2Fapi%2Fidezet%2FJn3%2C16%2FRUF"
   ```

### API returns empty or malformed data

Some proxies may modify the response. Try a different proxy service or set up your own.

### Rate limiting

Public CORS proxies may have rate limits. If you experience throttling:
1. Use the Tauri desktop mode instead
2. Set up your own proxy
3. Add request caching/debouncing

## Best Practices

1. **Use Tauri mode for production** - No CORS issues, better performance
2. **Browser mode is for development only** - Useful for quick testing without full Tauri build
3. **Don't expose sensitive data** - CORS proxies can see all request/response data
4. **Consider caching** - Reduce API calls to minimize proxy usage
5. **Have a fallback** - The app gracefully handles proxy failures

## File Locations

| File | Purpose |
|------|---------|
| `src/lib/config/bible-api.ts` | CORS proxy configuration |
| `src/lib/utils/bible-api.ts` | API service with proxy logic |
| `src-tauri/src/bible.rs` | Rust handlers (CORS-free) |
| `.env.example` | Environment variable template |

## Related Links

- [MDN: CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS)
- [Tauri HTTP Client](https://tauri.app/v1/api/js/http/)
- [szentiras.eu API](https://szentiras.eu/)
- [corsproxy.io](https://corsproxy.io/)
