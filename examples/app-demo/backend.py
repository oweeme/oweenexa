"""Backend de ejemplo — cualquier backend (PHP, Go, Node...) sirve igual.
Sesión por cookie HttpOnly: el JavaScript de la app nunca ve el token."""
from http.server import BaseHTTPRequestHandler, HTTPServer
from http.cookies import SimpleCookie
import json

USERS = [{"id": 1, "name": "Ana", "email": "ana@x.io"}, {"id": 2, "name": "Beto", "email": "beto@x.io"}, {"id": 3, "name": "Carla", "email": "carla@x.io"}]
SESSIONS = {}  # sid -> nombre


class H(BaseHTTPRequestHandler):
    def _cors(self):
        # Solo la app de escritorio llega de otro origen (tauri://localhost,
        # http://tauri.localhost en Windows): la web usa el proxy y no lo necesita.
        origin = self.headers.get("Origin")
        if origin and ("tauri" in origin):
            self.send_header("Access-Control-Allow-Origin", origin)
            self.send_header("Access-Control-Allow-Credentials", "true")
            self.send_header("Access-Control-Allow-Headers", "Content-Type")
            self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")

    def do_OPTIONS(self):
        self.send_response(204)
        self._cors()
        self.end_headers()

    def _json(self, status, body, cookie=None):
        self.send_response(status)
        self._cors()
        self.send_header("Content-Type", "application/json")
        if cookie:
            self.send_header("Set-Cookie", cookie)
        self.end_headers()
        self.wfile.write(json.dumps(body).encode())

    def _who(self):
        c = SimpleCookie(self.headers.get("Cookie", ""))
        return SESSIONS.get(c["sid"].value) if "sid" in c else None

    def do_POST(self):
        n = int(self.headers.get("Content-Length") or 0)
        data = json.loads(self.rfile.read(n) or b"{}")
        if self.path == "/api/login":
            name = data.get("name") or "anon"
            sid = f"s{len(SESSIONS) + 1}"
            SESSIONS[sid] = name
            self._json(200, {"name": name}, f"sid={sid}; HttpOnly; Path=/; SameSite=Lax")
        elif self.path == "/api/logout":
            self._json(200, {"ok": True}, "sid=; HttpOnly; Path=/; Max-Age=0")
        else:
            self._json(404, {"error": "no existe"})

    def do_GET(self):
        if self.path == "/api/catalog":  # público: lo usa una página de Nexa Core en build
            return self._json(200, {"title": "Equipos", "groups": [
                {"name": "Diseño", "members": [{"name": "Ana", "role": "lead"}, {"name": "Beto", "role": "dev"}]},
                {"name": "Datos", "members": [{"name": "Carla", "role": "lead"}]},
            ]})
        who = self._who()
        if self.path == "/api/me":
            return self._json(200, {"name": who}) if who else self._json(401, {"error": "sin sesión"})
        if not who:
            return self._json(401, {"error": "sin sesión"})
        if self.path == "/api/users":
            return self._json(200, USERS)
        if self.path.startswith("/api/users/"):
            found = [u for u in USERS if str(u["id"]) == self.path.split("/")[-1]]
            return self._json(200, found[0]) if found else self._json(404, {"error": "no existe"})
        self._json(404, {"error": "no existe"})

    def log_message(self, *a):
        pass


HTTPServer(("127.0.0.1", 9100), H).serve_forever()
