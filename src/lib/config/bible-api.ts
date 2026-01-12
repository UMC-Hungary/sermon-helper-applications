export interface BibleApiConfig {
  v2ApiUrl: string;
  legacyApiUrl: string;
  // Browser-only proxy URLs (used when not in Tauri to bypass CORS)
  browserProxyUrl?: string;
  useCorsProxy: boolean;
}

const DEFAULT_CONFIG: BibleApiConfig = {
  v2ApiUrl: 'https://api.nyiregyhazimetodista.hu',
  legacyApiUrl: 'https://szentiras.eu',
  // Default CORS proxy for browser mode (can be overridden via env)
  // Options:
  //   - https://corsproxy.io/?
  //   - https://api.allorigins.win/raw?url=
  //   - Your own proxy server
  browserProxyUrl: 'https://corsproxy.io/?',
  useCorsProxy: true,
};

// Get API configuration from environment variables or use defaults
export function getApiConfig(): BibleApiConfig {
  // Check if running in browser environment with import.meta.env
  if (typeof import.meta !== 'undefined' && import.meta.env) {
    return {
      v2ApiUrl: (import.meta.env.VITE_BIBLE_V2_API_URL as string) || DEFAULT_CONFIG.v2ApiUrl,
      legacyApiUrl: (import.meta.env.VITE_BIBLE_LEGACY_API_URL as string) || DEFAULT_CONFIG.legacyApiUrl,
      browserProxyUrl: (import.meta.env.VITE_CORS_PROXY_URL as string) || DEFAULT_CONFIG.browserProxyUrl,
      useCorsProxy: import.meta.env.VITE_USE_CORS_PROXY !== 'false',
    };
  }

  return DEFAULT_CONFIG;
}

/**
 * Build a proxied URL for browser mode
 * @param url The original URL to proxy
 * @param proxyUrl The proxy base URL
 * @returns The proxied URL
 */
export function buildProxiedUrl(url: string, proxyUrl?: string): string {
  if (!proxyUrl) {
    return url;
  }
  // Most CORS proxies expect the full URL as a parameter
  return `${proxyUrl}${encodeURIComponent(url)}`;
}

// Export default config for direct access
export const defaultApiConfig = DEFAULT_CONFIG;
