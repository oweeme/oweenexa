#!/bin/bash
# Prueba de punta a punta del renderizado bajo demanda de `nexa bundle` (examples/blog-store-demo).
# Requiere: api.py en :9300 y el ejecutable en :4500 (ver el README del ejemplo).
B=http://127.0.0.1:4500
API=http://127.0.0.1:9300
pass=0; fail=0
ok() { if [ "$2" = "1" ]; then echo "OK   $1"; pass=$((pass+1)); else echo "FAIL $1"; fail=$((fail+1)); fi; }
code() { curl -s -o /dev/null -w "%{http_code}" "$@"; }
hdr()  { curl -sI "$1" | tr -d '\r' | grep -i "^$2:" | cut -d' ' -f2-; }
count() { curl -s $API/_log | python3 -c "import sys,json; l=json.load(sys.stdin); print(sum(1 for x in l if x==sys.argv[1]))" "$1"; }

# 1. Pre-renderizado: la ruta del usuario existe sin tocar la API
P=/blog/hectorarchitect/desarrollando-oweelinux-conlinux
ok "ruta del usuario pre-renderizada: 200" $([ "$(code $B$P)" = 200 ] && echo 1 || echo 0)
ok "lo pre-renderizado no pasa por el renderizador (sin X-Nexa-Cache)" $([ -z "$(hdr $B$P x-nexa-cache)" ] && echo 1 || echo 0)
ok "contenido real de la API en el HTML" $(curl -s $B$P | grep -q "Desarrollando OweeLinux con Linux" && echo 1 || echo 0)

# 2. Post publicado DESPUÉS del build
ok "post nuevo antes de publicar: 404" $([ "$(code $B/blog/hectorarchitect/nuevo-post)" = 404 ] && echo 1 || echo 0)
curl -s $API/_publish >/dev/null
sleep 11   # el 404 se recuerda máx. 10 s
ok "post publicado tras el build: 200 sin recompilar" $([ "$(code $B/blog/hectorarchitect/nuevo-post)" = 200 ] && echo 1 || echo 0)
ok "título del post nuevo en el HTML" $(curl -s $B/blog/hectorarchitect/nuevo-post | grep -q "Post recién publicado" && echo 1 || echo 0)

# 3. Caché
N=/blog/hectorarchitect/nuevo-post
curl -s -o /dev/null $B$N
ok "segunda visita: X-Nexa-Cache hit" $([ "$(hdr $B$N x-nexa-cache)" = "hit" ] && echo 1 || echo 0)
before=$(count /api/blog/hectorarchitect/nuevo-post)
for i in 1 2 3 4 5; do curl -s -o /dev/null $B$N; done
after=$(count /api/blog/hectorarchitect/nuevo-post)
ok "5 visitas dentro de la ventana: 0 llamadas nuevas a la API" $([ "$before" = "$after" ] && echo 1 || echo 0)
curl -s $API/_edit >/dev/null
sleep 3
ok "tras revalidar (2 s) se ve el post EDITADO" $(curl -s $B$N | grep -q "Post EDITADO" && echo 1 || echo 0)

# 4. i18n + tienda dinámica
ok "idioma por defecto sin endpoint: /es" $([ "$(code $B/es)" = 200 ] && echo 1 || echo 0)
ok "idioma inexistente /xx: 404" $([ "$(code $B/xx)" = 404 ] && echo 1 || echo 0)
ok "idioma inexistente en una ruta profunda: 404" $([ "$(code $B/xx/shop/zapatos/runner)" = 404 ] && echo 1 || echo 0)
ok "producto nuevo (en inglés) renderizado bajo demanda" $([ "$(code $B/en/shop/zapatos/trail)" = 200 ] && echo 1 || echo 0)
ok "...con textos en inglés ('Price')" $(curl -s $B/en/shop/zapatos/trail | grep -q "Price: 120" && echo 1 || echo 0)
ok "...y la clave que falta en inglés cae al español (fallback)" $(curl -s $B/en/shop/zapatos/trail | grep -q "Software libre" && echo 1 || echo 0)
ok "hreflang presente en la página dinámica" $(curl -s $B/en/shop/zapatos/trail | grep -q 'hreflang="es"' && echo 1 || echo 0)

# 5. Seguridad
curl -s -o /dev/null "$B/blog/hectorarchitect/a%3Fx%3D1%26admin%3D1"
ok "inyección de query en el slug: la API recibió el valor CODIFICADO" $(curl -s $API/_log | grep -q '/api/blog/hectorarchitect/a%3Fx%3D1%26admin%3D1' && echo 1 || echo 0)
ok "...y nunca recibió un query inyectado" $(curl -s $API/_log | grep -q 'admin=1' && echo 0 || echo 1)
ok "path traversal: 400" $([ "$(code --path-as-is $B/blog/../../etc/passwd)" = 400 ] && echo 1 || echo 0)
calls_before=$(curl -s $API/_log | python3 -c "import sys,json; print(len(json.load(sys.stdin)))")
for u in /wp-login.php /.env /xmlrpc.php /favicon.ico; do curl -s -o /dev/null $B$u; done
calls_after=$(curl -s $API/_log | python3 -c "import sys,json; print(len(json.load(sys.stdin)))")
ok "rutas que no parecen páginas (.php, .env...) no cuestan ninguna llamada a la API" $([ "$calls_before" = "$calls_after" ] && echo 1 || echo 0)

# 6. Avalancha: 60 visitas simultáneas a un producto vencido/nuevo
curl -s $API/_publish >/dev/null
seq 1 60 | xargs -P 30 -I{} curl -s -o /dev/null $B/en/shop/zapatos/trail
sleep 3
seq 1 60 | xargs -P 30 -I{} curl -s -o /dev/null $B/en/shop/zapatos/trail
n=$(count /api/shop/zapatos/trail)
echo "     (llamadas a la API para ese producto tras 120 visitas con 2 ventanas de caché: $n)"
ok "avalancha: 120 visitas = unas pocas llamadas (<=4), no 120" $([ "$n" -le 4 ] && echo 1 || echo 0)

# 7. La API cae: se sirve la última copia
ps -eo pid,args | grep "api.py" | grep -v grep | awk '{print $1}' | xargs -r kill
sleep 3
ok "API caída: el post ya visto se sigue sirviendo (stale)" $([ "$(code $B$N)" = 200 ] && [ "$(hdr $B$N x-nexa-cache)" = "stale" ] && echo 1 || echo 0)
ok "API caída y ruta nunca vista: 502 (no cuelga)" $([ "$(code $B/blog/otro/jamas-visto)" = 502 ] && echo 1 || echo 0)
ok "API caída: lo pre-renderizado sigue perfecto" $([ "$(code $B$P)" = 200 ] && echo 1 || echo 0)
ok "502 sin filtrar la URL interna de la API" $(curl -s $B/blog/otro/jamas-visto2 | grep -q "9300" && echo 0 || echo 1)

echo; echo "RESULTADO: $pass OK, $fail FAIL"
