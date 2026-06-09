type CollectorFetchResponse = Pick<Response, "ok" | "status" | "statusText" | "json">;

export type CollectorFetcher = (
  input: string,
  init?: RequestInit
) => Promise<CollectorFetchResponse>;

export function createCollectorFetcher(
  apiUrl?: string | null,
  baseFetch: CollectorFetcher = fetch
): CollectorFetcher {
  return (input, init) => baseFetch(resolveCollectorUrl(input, apiUrl), init);
}

export function resolveCollectorUrl(input: string, apiUrl?: string | null): string {
  if (!apiUrl || isAbsoluteUrl(input)) {
    return input;
  }
  if (!input.startsWith("/api") && !input.startsWith("/screenshots")) {
    return input;
  }
  return `${apiUrl.replace(/\/+$/, "")}${input}`;
}

function isAbsoluteUrl(value: string): boolean {
  return /^[a-z][a-z\d+\-.]*:\/\//i.test(value);
}
