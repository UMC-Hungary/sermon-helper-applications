import { get } from 'svelte/store';
import { serverUrl, authToken } from '$lib/stores/server-url.js';

async function apiFetch<T>(path: string, options: RequestInit = {}): Promise<T> {
  const base = get(serverUrl);
  const token = get(authToken);

  const res = await fetch(`${base}${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
      ...options.headers,
    },
  });

  if (!res.ok) {
    throw new Error(`API error ${res.status}: ${await res.text()}`);
  }

  return res.json() as Promise<T>;
}

export { apiFetch };
