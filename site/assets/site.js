/* Comportamiento del sitio: tema, idioma (portada), menú móvil, copiar código, índice de la página y buscador. */
(function () {
  var root = document.documentElement;
  var base = root.getAttribute("data-base") || "./";
  function get(k) { try { return localStorage.getItem(k); } catch (e) { return null; } }
  function set(k, v) { try { localStorage.setItem(k, v); } catch (e) {} }
  function $(s, c) { return (c || document).querySelector(s); }
  function $$(s, c) { return Array.prototype.slice.call((c || document).querySelectorAll(s)); }

  /* ---- tema ---- */
  var saved = get("nexa-theme");
  if (saved === "light" || saved === "dark") root.setAttribute("data-theme", saved);
  var themeBtn = $("#theme");
  if (themeBtn) themeBtn.addEventListener("click", function () {
    var dark = root.getAttribute("data-theme") === "dark" ||
      (!root.getAttribute("data-theme") && matchMedia("(prefers-color-scheme: dark)").matches);
    var next = dark ? "light" : "dark";
    root.setAttribute("data-theme", next); set("nexa-theme", next);
  });

  /* ---- idioma (solo la portada es bilingüe) ---- */
  var langBtn = $("#lang");
  if (langBtn) {
    var lang = get("nexa-lang") || ((navigator.language || "es").slice(0, 2) === "es" ? "es" : "en");
    var setLang = function (l) {
      root.setAttribute("data-lang", l); root.lang = l; set("nexa-lang", l);
      langBtn.textContent = l === "es" ? "EN" : "ES";
      $$(".copy").forEach(function (b) { b.textContent = l === "es" ? "Copiar" : "Copy"; });
    };
    setLang(lang);
    langBtn.addEventListener("click", function () { setLang(root.getAttribute("data-lang") === "es" ? "en" : "es"); });
  }

  /* ---- menú lateral en móvil ---- */
  var menu = $("#menu"), side = $(".side");
  if (menu && side) {
    menu.addEventListener("click", function () { side.classList.toggle("on"); });
    side.addEventListener("click", function (e) { if (e.target.tagName === "A") side.classList.remove("on"); });
  }

  /* ---- copiar código ---- */
  $$(".copy").forEach(function (b) {
    b.addEventListener("click", function () {
      var pre = b.closest(".code").querySelector("pre");
      var l = root.getAttribute("data-lang") || "es";
      var done = function () {
        b.textContent = l === "es" ? "¡Copiado!" : "Copied!";
        setTimeout(function () { b.textContent = l === "es" ? "Copiar" : "Copy"; }, 1500);
      };
      if (navigator.clipboard && navigator.clipboard.writeText) navigator.clipboard.writeText(pre.innerText).then(done, function () {});
    });
  });

  /* ---- índice de la página: resalta la sección visible ---- */
  var links = $$(".toc a");
  if (links.length && "IntersectionObserver" in window) {
    var byId = {};
    links.forEach(function (a) { byId[a.getAttribute("href").slice(1)] = a; });
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (en) {
        if (en.isIntersecting && byId[en.target.id]) {
          links.forEach(function (a) { a.classList.remove("on"); });
          byId[en.target.id].classList.add("on");
        }
      });
    }, { rootMargin: "-70px 0px -70% 0px" });
    $$(".prose h2[id], .prose h3[id]").forEach(function (h) { io.observe(h); });
  }

  /* ---- buscador ---- */
  var box = $("#buscar"), input = box && $("input", box), list = box && $("ul", box), index = null, sel = 0, hits = [];
  function esc(s) { return s.replace(/[&<>"]/g, function (c) { return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]; }); }
  function norm(s) { return s.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, ""); }
  function open() {
    if (!box) return;
    box.classList.add("on"); input.value = ""; input.focus(); render([]);
    if (!index) fetch(base + "search-index.json").then(function (r) { return r.json(); }).then(function (j) { index = j; });
  }
  function close() { if (box) box.classList.remove("on"); }
  function search(q) {
    if (!index || !q.trim()) return [];
    var terms = norm(q).split(/\s+/).filter(Boolean), out = [];
    index.forEach(function (e) {
      var hay = norm(e.t), body = norm(e.x || ""), score = 0;
      for (var i = 0; i < terms.length; i++) {
        var t = terms[i];
        if (hay.indexOf(t) === 0) score += 12; else if (hay.indexOf(t) > -1) score += 8;
        else if (body.indexOf(t) > -1) score += 2; else { score = 0; break; }
      }
      if (score) out.push({ e: e, s: score + (e.h ? 0 : 3) });
    });
    return out.sort(function (a, b) { return b.s - a.s; }).slice(0, 12).map(function (o) { return o.e; });
  }
  function render(res) {
    hits = res; sel = 0;
    if (!res.length) { list.innerHTML = '<li class="vacio">' + (input.value.trim() ? "Sin resultados" : "Escribe para buscar en toda la documentación…") + "</li>"; return; }
    list.innerHTML = res.map(function (e, i) {
      return '<li class="' + (i === 0 ? "sel" : "") + '"><a href="' + base + e.u + '">' + esc(e.t) + "<small>" + esc(e.p) + "</small></a></li>";
    }).join("");
  }
  function move(d) {
    var items = $$("li", list); if (!items.length || !hits.length) return;
    items[sel].classList.remove("sel"); sel = (sel + d + items.length) % items.length;
    items[sel].classList.add("sel"); items[sel].scrollIntoView({ block: "nearest" });
  }
  if (box) {
    $$("[data-search]").forEach(function (b) { b.addEventListener("click", open); });
    input.addEventListener("input", function () { render(search(input.value)); });
    box.addEventListener("click", function (e) { if (e.target === box) close(); });
    document.addEventListener("keydown", function (e) {
      var typing = /INPUT|TEXTAREA/.test((document.activeElement || {}).tagName || "");
      if ((e.key === "k" && (e.ctrlKey || e.metaKey)) || (e.key === "/" && !typing)) { e.preventDefault(); open(); }
      else if (!box.classList.contains("on")) return;
      else if (e.key === "Escape") close();
      else if (e.key === "ArrowDown") { e.preventDefault(); move(1); }
      else if (e.key === "ArrowUp") { e.preventDefault(); move(-1); }
      else if (e.key === "Enter" && hits[sel]) location.href = base + hits[sel].u;
    });
  }
})();
