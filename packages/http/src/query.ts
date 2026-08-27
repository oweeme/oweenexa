import { state, type Signal } from "@nexa/reactivity";

/**
 * - `cache-first`: si hay algo en caché, se usa y no se vuelve a pedir
 *   solo (hace falta `refetch()` explícito).
 * - `network-first`: siempre espera a la red antes de mostrar nada; si la
 *   red falla, cae a lo último que hubiera en caché.
 * - `stale-while-revalidate`: muestra de inmediato lo que haya en caché
 *   (sin `loading`) mientras revalida en segundo plano.
 */
export type CachePolicy = "cache-first" | "network-first" | "stale-while-revalidate";

interface CacheEntry {
    data: unknown;
}

const defaultCache = new Map<string, CacheEntry>();

export interface QueryOptions<T> {
    key: string;
    fetch: () => Promise<T>;
    cachePolicy?: CachePolicy;
    /** Inyectable para tests / para aislar cachés entre partes de la app. */
    cache?: Map<string, CacheEntry>;
}

export interface QueryResult<T> {
    data: Signal<T | undefined>;
    loading: Signal<boolean>;
    error: Signal<unknown>;
    refetch: () => Promise<void>;
}

export function query<T>(options: QueryOptions<T>): QueryResult<T> {
    const { key, fetch, cachePolicy = "cache-first" } = options;
    const cache = options.cache ?? defaultCache;
    const cached = cache.get(key);

    const startsWithCachedData = cachePolicy !== "network-first" && cached !== undefined;
    const data = state<T | undefined>(startsWithCachedData ? (cached!.data as T) : undefined);
    const loading = state(!startsWithCachedData);
    const error = state<unknown>(undefined);

    async function run(): Promise<void> {
        try {
            const result = await fetch();
            cache.set(key, { data: result });
            data.value = result;
            error.value = undefined;
        } catch (err) {
            // Un fallo de red no debe borrar el último dato bueno conocido:
            // si `data` sigue vacío (ej. "network-first" que aún no había
            // mostrado nada) pero hay algo en caché, se cae a eso.
            if (data.value === undefined && cached !== undefined) {
                data.value = cached.data as T;
            }
            error.value = err;
        } finally {
            loading.value = false;
        }
    }

    const needsFetch = cachePolicy !== "cache-first" || cached === undefined;
    if (needsFetch) void run();

    return { data, loading, error, refetch: run };
}
