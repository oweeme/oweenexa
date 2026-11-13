import { describe, expect, it, vi } from "vitest";

import { appInfo, invoke } from "../src/native";

describe("native.invoke (Fase 63)", () => {
    it("llama al comando de Rust con sus argumentos y devuelve el resultado", async () => {
        const core = { invoke: vi.fn().mockResolvedValue({ ok: true }) };
        const env = { isTauri: true, __TAURI__: { core } };
        await expect(invoke("guardar", { id: 7 }, env)).resolves.toEqual({ ok: true });
        expect(core.invoke).toHaveBeenCalledWith("guardar", { id: 7 });
    });

    it("fuera de Tauri lanza un error claro (no finge funcionar)", async () => {
        await expect(invoke("x", undefined, {})).rejects.toThrow(/solo funciona dentro de la app de escritorio/);
        await expect(invoke("x", undefined, { Capacitor: { isNativePlatform: () => true } })).rejects.toThrow(/Tauri/);
    });

    it("en Tauri sin withGlobalTauri explica cómo arreglarlo", async () => {
        await expect(invoke("x", undefined, { isTauri: true })).rejects.toThrow(/withGlobalTauri/);
    });

    it("un error de Rust se propaga tal cual", async () => {
        const env = { isTauri: true, __TAURI__: { core: { invoke: vi.fn().mockRejectedValue("permiso denegado") } } };
        await expect(invoke("x", undefined, env)).rejects.toBe("permiso denegado");
    });

    it("appInfo invoca el comando `app_info`", async () => {
        const core = { invoke: vi.fn().mockResolvedValue({ name: "demo", version: "1", os: "linux", arch: "x86_64" }) };
        const info = await appInfo({ isTauri: true, __TAURI__: { core } });
        expect(info.os).toBe("linux");
        expect(core.invoke).toHaveBeenCalledWith("app_info", undefined);
    });
});
