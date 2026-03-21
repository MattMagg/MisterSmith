export function formatTimestamp(value?: string | null): string {
  if (!value) {
    return 'not available';
  }

  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }

  return parsed.toLocaleString();
}

export function prettyJson(value: unknown): string {
  if (value === undefined) {
    return 'undefined';
  }

  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

export function readMetric(
  source: Record<string, unknown> | null | undefined,
  key: string,
): string {
  if (!source || !(key in source)) {
    return 'n/a';
  }

  const value = source[key];
  return typeof value === 'string' || typeof value === 'number'
    ? String(value)
    : prettyJson(value);
}

export function formatError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
}
