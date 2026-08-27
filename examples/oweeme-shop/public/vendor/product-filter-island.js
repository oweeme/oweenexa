// packages/reactivity/src/reactive.ts
var activeEffect = null;
var pending = /* @__PURE__ */ new Set();
var flushScheduled = false;
function schedule(effect2) {
  pending.add(effect2);
  if (flushScheduled) return;
  flushScheduled = true;
  queueMicrotask(flush);
}
function flush() {
  flushScheduled = false;
  const effects = Array.from(pending);
  pending.clear();
  for (const effect2 of effects) {
    effect2.execute();
  }
}
var Signal = class {
  #value;
  #subscribers = /* @__PURE__ */ new Set();
  constructor(initial) {
    this.#value = initial;
  }
  get value() {
    if (activeEffect) {
      this.#subscribers.add(activeEffect);
      activeEffect.deps.add(this);
    }
    return this.#value;
  }
  set value(next) {
    if (Object.is(next, this.#value)) return;
    this.#value = next;
    for (const effect2 of this.#subscribers) {
      schedule(effect2);
    }
  }
  /** Lee el valor sin suscribirse: no crea una dependencia. */
  peek() {
    return this.#value;
  }
  /** Usado por los efectos al limpiar sus dependencias antes de re-ejecutarse. */
  unsubscribe(effect2) {
    this.#subscribers.delete(effect2);
  }
};
function state(initial) {
  return new Signal(initial);
}
var EffectImpl = class {
  deps = /* @__PURE__ */ new Set();
  #fn;
  #disposed = false;
  constructor(fn) {
    this.#fn = fn;
    this.execute();
  }
  execute() {
    if (this.#disposed) return;
    this.#cleanup();
    const previous = activeEffect;
    activeEffect = this;
    try {
      this.#fn();
    } finally {
      activeEffect = previous;
    }
  }
  #cleanup() {
    for (const dep of this.deps) {
      dep.unsubscribe(this);
    }
    this.deps.clear();
  }
  dispose() {
    this.#disposed = true;
    this.#cleanup();
    pending.delete(this);
  }
};
function createEffect(fn) {
  const instance = new EffectImpl(fn);
  return () => instance.dispose();
}
var effect = Object.assign(createEffect, { client: createEffect });

// examples/oweeme-shop/src/islands/productFilter.island.ts
function isProduct(value) {
  return !!value && typeof value === "object" && typeof value.slug === "string" && typeof value.name === "string" && typeof value.price === "number";
}
function mount(el, props) {
  const products = Array.isArray(props.products) ? props.products.filter(isProduct) : [];
  const input = document.createElement("input");
  input.type = "search";
  input.className = "nx-input";
  input.placeholder = "Buscar productos\u2026";
  const list = document.createElement("ul");
  el.innerHTML = "";
  el.append(input, list);
  const query = state("");
  const onInput = () => {
    query.value = input.value;
  };
  input.addEventListener("input", onInput);
  const disposeEffect = effect(() => {
    const needle = query.value.trim().toLowerCase();
    const visible = needle ? products.filter((p) => p.name.toLowerCase().includes(needle)) : products;
    list.innerHTML = "";
    for (const product of visible) {
      const item = document.createElement("li");
      item.textContent = `${product.name} \u2014 $${product.price}`;
      list.appendChild(item);
    }
    if (visible.length === 0) {
      const empty = document.createElement("li");
      empty.textContent = "Sin resultados.";
      list.appendChild(empty);
    }
  });
  return () => {
    input.removeEventListener("input", onInput);
    disposeEffect();
  };
}
export {
  mount as default
};
