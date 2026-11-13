from http.server import BaseHTTPRequestHandler, HTTPServer, ThreadingHTTPServer
import json, threading
LOG = []; LOCK = threading.Lock()
POSTS = {("hectorarchitect", "desarrollando-oweelinux-conlinux"): {"title": "Desarrollando OweeLinux con Linux", "body": "v1"}}
SHOP = {("zapatos", "runner"): {"name": "Runner", "price": 99}}
class H(BaseHTTPRequestHandler):
    def _j(self, s, b):
        self.send_response(s); self.send_header("Content-Type", "application/json"); self.end_headers(); self.wfile.write(json.dumps(b).encode())
    def do_GET(self):
        p = self.path
        if p == "/_log":
            with LOCK: return self._j(200, list(LOG))
        with LOCK: LOG.append(p)
        if p == "/api/articles": return self._j(200, [{"slug": "demo"}])
        if p == "/api/articles/demo":
            return self._j(200, {
                "title": "Artículo con contenido enriquecido",
                # HTML de un editor (el autor) MÁS ataques: ninguno debe ejecutarse.
                "html": "<h2>Subtítulo</h2><p>Texto con <strong>negrita</strong> y un <a href=\"https://oweeme.com/blog\">enlace</a>.</p>"
                        "<ul><li>uno</li><li>dos</li></ul><pre><code class=\"language-rust\">fn main() {}</code></pre>"
                        "<img src=\"https://cdn.example.com/a.png\" alt=\"foto\">"
                        "<script>window.__pwned=1</script><img src=x onerror=\"window.__pwned=1\">"
                        "<a id=\"evil\" href=\"javascript:window.__pwned=1\">no me hagas clic</a>"
                        "<svg onload=\"window.__pwned=1\"></svg><iframe srcdoc=\"<script>parent.__pwned=1</script>\"></iframe>"
                        "<p onclick=\"window.__pwned=1\" style=\"position:fixed\">p</p>",
                "markdown": "# Título MD\n\nUn párrafo con **negrita**, `código` y [enlace](https://oweeme.com).\n\n"
                            "- a\n- b\n\n| col | val |\n|---|---|\n| x | 1 |\n\n"
                            "[malo](javascript:window.__pwned=1)\n\n<script>window.__pwned=1</script>\n\n<img src=x onerror=\"window.__pwned=1\">",
            })
        if p == "/api/blog": return self._j(200, [{"author": a, "slug": s} for (a, s) in POSTS])
        if p.startswith("/api/blog/"):
            _, _, _, a, s = p.split("/"); v = POSTS.get((a, s)); return self._j(200, v) if v else self._j(404, {"e": 1})
        if p == "/api/shop": return self._j(200, [{"category": c, "product": x} for (c, x) in SHOP])
        if p.startswith("/api/shop/"):
            _, _, _, c, x = p.split("/"); v = SHOP.get((c, x)); return self._j(200, v) if v else self._j(404, {"e": 1})
        if p == "/_publish":
            POSTS[("hectorarchitect", "nuevo-post")] = {"title": "Post recién publicado", "body": "v1"}
            SHOP[("zapatos", "trail")] = {"name": "Trail", "price": 120}
            return self._j(200, {"ok": 1})
        if p == "/_edit":
            POSTS[("hectorarchitect", "nuevo-post")] = {"title": "Post EDITADO", "body": "v2"}; return self._j(200, {"ok": 1})
        self._j(404, {"e": 1})
    def log_message(self, *a): pass
ThreadingHTTPServer(("127.0.0.1", 9300), H).serve_forever()
