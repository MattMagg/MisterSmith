import type { StoredSettings, TabId } from './types';

const SETTINGS_STORAGE_KEY = 'mister-smith-operator-console-settings';
const DEFAULT_RUNTIME_BASE_URL = 'http://127.0.0.1:8080';
const DEFAULT_NATS_MONITOR_URL = 'http://127.0.0.1:8222';

const TAB_IDS = new Set<TabId>(['runs', 'sessions', 'agents', 'health']);

export const DEFAULT_SETTINGS: StoredSettings = {
  runtimeBaseUrl: DEFAULT_RUNTIME_BASE_URL,
  natsMonitorUrl: DEFAULT_NATS_MONITOR_URL,
  reconnectEnabled: true,
  activeTab: 'runs',
};

export function loadStoredSettings(): StoredSettings {
  if (typeof window === 'undefined' || !window.localStorage) {
    return DEFAULT_SETTINGS;
  }

  const raw = window.localStorage.getItem(SETTINGS_STORAGE_KEY);
  if (!raw) {
    return DEFAULT_SETTINGS;
  }

  try {
    return normalizeStoredSettings(JSON.parse(raw));
  } catch {
    return DEFAULT_SETTINGS;
  }
}

export function saveStoredSettings(settings: StoredSettings): void {
  if (typeof window === 'undefined' || !window.localStorage) {
    return;
  }

  window.localStorage.setItem(
    SETTINGS_STORAGE_KEY,
    JSON.stringify(normalizeStoredSettings(settings)),
  );
}

export function normalizeStoredSettings(value: unknown): StoredSettings {
  const candidate = isObject(value) ? value : {};
  const activeTab = isTabId(candidate.activeTab) ? candidate.activeTab : 'runs';

  return {
    runtimeBaseUrl: normalizeUrl(candidate.runtimeBaseUrl, DEFAULT_RUNTIME_BASE_URL),
    natsMonitorUrl: normalizeUrl(candidate.natsMonitorUrl, DEFAULT_NATS_MONITOR_URL),
    reconnectEnabled:
      typeof candidate.reconnectEnabled === 'boolean'
        ? candidate.reconnectEnabled
        : true,
    activeTab,
  };
}

export function normalizeUrl(value: unknown, fallback: string): string {
  if (typeof value !== 'string') {
    return fallback;
  }

  const trimmed = value.trim();
  if (!trimmed) {
    return fallback;
  }

  return trimmed.replace(/\/+$/, '');
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function isTabId(value: unknown): value is TabId {
  return typeof value === 'string' && TAB_IDS.has(value as TabId);
}
