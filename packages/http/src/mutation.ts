import { state, type Signal } from "@nexa/reactivity";

export interface MutationOptions<Args extends unknown[], T> {
    execute: (...args: Args) => Promise<T>;
}

export interface MutationResult<Args extends unknown[], T> {
    data: Signal<T | undefined>;
    loading: Signal<boolean>;
    error: Signal<unknown>;
    execute: (...args: Args) => Promise<T>;
}

/**
 * A diferencia de `query()`, una mutación nunca se dispara sola: solo
 * cuando algo (típicamente un `onClick`) llama a `execute(...)`.
 */
export function mutation<Args extends unknown[], T>(
    options: MutationOptions<Args, T>,
): MutationResult<Args, T> {
    const data = state<T | undefined>(undefined);
    const loading = state(false);
    const error = state<unknown>(undefined);

    async function execute(...args: Args): Promise<T> {
        loading.value = true;
        error.value = undefined;
        try {
            const result = await options.execute(...args);
            data.value = result;
            return result;
        } catch (err) {
            error.value = err;
            throw err;
        } finally {
            loading.value = false;
        }
    }

    return { data, loading, error, execute };
}
