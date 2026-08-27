/** Cómo pedir el número de versión actual del servidor de `nexa dev`. */
export type VersionFetcher = () => Promise<string>;

export const defaultVersionFetcher: VersionFetcher = () =>
    fetch("/__nexa_dev__/version", { cache: "no-store" }).then((res) => res.text());
