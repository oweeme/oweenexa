export { h, Fragment, insert, dynamic } from "./h";
export type { Child, Props, Component } from "./h";
export { Show, For, Switch, Match } from "./control";
export { startApp, navigate, route, matchPattern } from "./router";
export type { AppRoute, PageProps, Guard, StartAppOptions } from "./router";
export { effect, onCleanup, createScope, runInScope, disposeScope } from "./scope";
export { state, computed, watch, Signal } from "@nexa/reactivity";
export type { ReadonlySignal } from "@nexa/reactivity";
export { persisted } from "./persisted";
// Capa de datos: el mismo cliente/queries/mutaciones de `@nexa/http`,
// re-exportados para importar todo desde un único lugar en el Modo App.
export { createApi, HttpError, query, mutation, connectSSE, connectSocket } from "@nexa/http";
export type { HttpClient, HttpClientOptions, QueryResult, MutationResult } from "@nexa/http";
