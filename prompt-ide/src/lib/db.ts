import type { ApiKeys } from './types';

const API_KEYS_STORAGE_KEY = 'inkwell-api-keys';

export function getApiKeys(): ApiKeys {
  try {
    const raw = localStorage.getItem(API_KEYS_STORAGE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

export function setApiKeys(keys: ApiKeys): void {
  localStorage.setItem(API_KEYS_STORAGE_KEY, JSON.stringify(keys));
}
