import { loadStripe, type Stripe } from "@stripe/stripe-js";

/**
 * Referencia de "paquete de comunidad" (Fase 15): un envoltorio real y
 * delgado sobre el SDK real de Stripe (`@stripe/stripe-js`, npm), no
 * escrito ni mantenido por el core de Nexa — exactamente lo que
 * `@nexa/stripe`, `@nexa/firebase`, etc. serían en la práctica. Existe
 * para probar que el mecanismo de imports de la Fase 15
 * (`nexa.toml` → `[imports]` → import map real en `<head>`) funciona
 * con código de un tercero, no solo con `@nexa/platform`.
 *
 * La forma (`stripe.load(...)`, no `loadStripe(...)` a secas) es
 * deliberada: `nexa-activation` detecta `<nombre>.` en el código de un
 * handler para decidir si antepone su import — un identificador llamado
 * a secas (sin `.`) no dispara nada.
 */
export const stripe = {
    load: (publishableKey: string): Promise<Stripe | null> => loadStripe(publishableKey),
};
