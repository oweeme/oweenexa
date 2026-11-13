// Generado por `nexa create` — tipos SOLO para el editor (autocompletado y
// sin subrayados rojos en el TSX). El compilador de Nexa no los lee: puedes
// borrar este archivo sin afectar `nexa build`.

declare namespace JSX {
    type Element = Node;
    interface IntrinsicElements {
        [tag: string]: Record<string, any>;
    }
    interface ElementChildrenAttribute {
        children: unknown;
    }
}

// ── Nexa Core (src/pages): nombres que el compilador reconoce al analizar ──
declare const data: any;
declare const params: Record<string, string>;
declare function t(key: string, args?: Record<string, unknown>): string;
// `data` es `any`, así que los items de un <For> también (un bucle anidado:
// `each={group.members}`).
declare function For(props: { each: any; children: (item: any) => unknown }): JSX.Element;
declare function Show(props: { when: unknown; fallback?: unknown; children?: unknown }): JSX.Element;
// Contenido enriquecido de tu API, SANEADO al renderizar (ver docs/SEGURIDAD.md).
declare function SafeHtml(props: { html: string | null | undefined }): JSX.Element;
declare function Markdown(props: { text: string | null | undefined }): JSX.Element;

// ── Modo App (src/app): la API de @nexa/app ──
declare module "@nexa/app" {
    export interface Signal<T> {
        value: T;
        peek(): T;
    }
    export interface ReadonlySignal<T> {
        readonly value: T;
        peek(): T;
    }
    export function state<T>(initial: T): Signal<T>;
    export function computed<T>(fn: () => T): ReadonlySignal<T>;
    export function watch<T>(source: () => T, callback: (value: T, previous: T | undefined) => void): () => void;
    export function effect(fn: () => void): () => void;
    export function onCleanup(fn: () => void): void;
    export function persisted<T>(key: string, initial: T, storage?: "local" | "session"): Signal<T>;

    export type Dictionary = { [key: string]: string | Dictionary };
    export interface I18n {
        readonly locale: Signal<string>;
        readonly available: string[];
        t(key: string, args?: Record<string, string | number>): string;
        setLocale(locale: string): void;
        number(value: number, options?: Intl.NumberFormatOptions): string;
        currency(value: number, currency: string, options?: Intl.NumberFormatOptions): string;
        date(value: Date | number | string, options?: Intl.DateTimeFormatOptions): string;
    }
    export function createI18n(dictionaries: Record<string, Dictionary>, options?: { fallback?: string; storageKey?: string | null }): I18n;

    export type Child = unknown;
    export interface PageProps extends Record<string, unknown> {
        params: Record<string, string>;
        query: URLSearchParams;
    }
    export function Show(props: { when: unknown; fallback?: Child; children?: Child | Child[] }): Node;
    export function For<T>(props: {
        each: readonly T[] | null | undefined | Signal<readonly T[]> | (() => readonly T[] | null | undefined);
        fallback?: Child;
        children: (item: T, index: () => number) => Node;
    }): Node;
    export function Switch(props: { fallback?: Child; children?: Child | Child[] }): Node;
    export function Match(props: { when: unknown; children?: Child | Child[] }): Node;

    export function navigate(to: string, options?: { replace?: boolean }): Promise<void>;
    export const route: {
        path: Signal<string>;
        params: Signal<Record<string, string>>;
        query: Signal<URLSearchParams>;
    };

    export class HttpError extends Error {
        readonly status: number;
    }
    export interface HttpClient {
        get<T>(path: string): Promise<T>;
        post<T>(path: string, body?: unknown): Promise<T>;
        put<T>(path: string, body?: unknown): Promise<T>;
        patch<T>(path: string, body?: unknown): Promise<T>;
        delete<T>(path: string): Promise<T>;
    }
    export function createApi(options?: {
        baseURL?: string;
        headers?: Record<string, string> | (() => Record<string, string>);
        credentials?: RequestCredentials;
        onUnauthorized?: (error: HttpError) => void;
    }): HttpClient;
    export function query<T>(options: {
        key: string;
        fetch: () => Promise<T>;
        cachePolicy?: "cache-first" | "network-first" | "stale-while-revalidate";
    }): { data: Signal<T | undefined>; loading: Signal<boolean>; error: Signal<unknown>; refetch: () => Promise<void> };
    export function mutation<Args extends unknown[], T>(options: { execute: (...args: Args) => Promise<T> }): {
        data: Signal<T | undefined>;
        loading: Signal<boolean>;
        error: Signal<unknown>;
        execute: (...args: Args) => Promise<T>;
    };
}

// ── `platform`: web, escritorio (Tauri) y móvil (Capacitor) ──
declare module "platform" {
    export const platform: {
        isTauri(): boolean;
        isCapacitor(): boolean;
        isWeb(): boolean;
        native: {
            invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
            appInfo(): Promise<{ name: string; version: string; os: string; arch: string }>;
        };
        [feature: string]: any;
    };
}
