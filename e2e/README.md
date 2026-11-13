# e2e — pruebas en un Chrome real

Cada script recibe la URL base y sale con código ≠ 0 si algo falla. Funcionan igual contra
`nexa preview`, `nexa dev` o el ejecutable único de `nexa bundle` (así se comprobó la Fase 64).

```bash
cd e2e && npm install            # puppeteer-core (usa tu Chrome; CHROME=/ruta/al/chrome si no está en /usr/bin/google-chrome)

# 1) SPA con login por cookie + proxy + Core con componentes  (examples/app-demo)
cd ../examples/app-demo
python3 backend.py &                                   # backend de ejemplo en :9100
NEXA_API_URL=http://127.0.0.1:9100 nexa build
nexa preview --port 4321 &                             # o: nexa bundle && ./demo-server --port 4321
node ../../e2e/spa-auth.mjs        http://127.0.0.1:4321    # 13 comprobaciones
node ../../e2e/core-components.mjs http://127.0.0.1:4321    # 10 comprobaciones

# 2) Juego con Phaser  (examples/phaser-demo)
cd ../phaser-demo && npm install && npm run vendor && nexa build
nexa preview --port 4331 &
node ../../e2e/phaser.mjs http://127.0.0.1:4331             # 7 comprobaciones
```
