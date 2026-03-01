import { get } from 'svelte/store';
import type { z } from 'zod';
import { serverUrl, authToken } from '$lib/stores/server-url.js';

type RequestOptions = Omit<RequestInit, 'headers' | 'body'> & {
  body?: unknown;
};

export async function apiFetch<S extends z.ZodType>(
  path: string,
  schema: S,
  options: RequestOptions = {},
): Promise<z.infer<S>> {
  const base = get(serverUrl);
  const token = get(authToken);
  const { body, ...restOptions } = options;

  const init: RequestInit = {
    ...restOptions,
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
  };
  if (body !== undefined) {
    init.body = JSON.stringify(body);
  }

  const res = await fetch(`${base}${path}`, init);

  if (!res.ok) {
    throw new Error(`API error ${res.status}: ${await res.text()}`);
  }

  if (res.status === 204) {
    return schema.parse(undefined) as z.infer<S>;
  }
  const data: unknown = await res.json();
  return schema.parse(data) as z.infer<S>;
}
