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
