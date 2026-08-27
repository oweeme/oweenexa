/** Cliente HTTP propio de Nexa: no depende de Axios ni de ninguna otra librería. */

export class HttpError extends Error {
    constructor(
        public readonly status: number,
        message: string,
    ) {
        super(message || `HTTP ${status}`);
        this.name = "HttpError";
    }
}

export interface HttpClientOptions {
    baseURL?: string;
    /** Inyectable para tests; por defecto, el `fetch` global. */
    fetchImpl?: typeof fetch;
}

export interface HttpClient {
    get<T>(path: string): Promise<T>;
    post<T>(path: string, body?: unknown): Promise<T>;
    put<T>(path: string, body?: unknown): Promise<T>;
    delete<T>(path: string): Promise<T>;
}

export function createApi(options: HttpClientOptions = {}): HttpClient {
    const baseURL = options.baseURL ?? "";
    const fetchImpl = options.fetchImpl ?? fetch;

    async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
        const hasBody = body !== undefined;
        const response = await fetchImpl(baseURL + path, {
            method,
            headers: hasBody ? { "Content-Type": "application/json" } : undefined,
            body: hasBody ? JSON.stringify(body) : undefined,
        });

        if (!response.ok) {
            throw new HttpError(response.status, await safeText(response));
        }

        if (response.status === 204) {
            return undefined as T;
        }

        return (await response.json()) as T;
    }

    return {
        get: (path) => request("GET", path),
        post: (path, body) => request("POST", path, body),
        put: (path, body) => request("PUT", path, body),
        delete: (path) => request("DELETE", path),
    };
}

async function safeText(response: Response): Promise<string> {
    try {
        return await response.text();
    } catch {
        return "";
    }
}
