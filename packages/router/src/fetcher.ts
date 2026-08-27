/** Cómo pedir el HTML completo de otra página. Inyectable para tests. */
export type PageFetcher = (path: string) => Promise<string>;

export const defaultFetcher: PageFetcher = (path) => fetch(path).then((res) => res.text());
