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

// packages/reactivity/src/component.ts
function mountComponent(component, container, props) {
  const cleanups = [];
  const ctx = {
    onCleanup(fn) {
      cleanups.push(fn);
    },
    mount(child, childContainer, childProps) {
      cleanups.push(mountComponent(child, childContainer, childProps));
    }
  };
  component(container, props, ctx);
  let disposed = false;
  return () => {
    if (disposed) return;
    disposed = true;
    for (const cleanup of cleanups.splice(0).reverse()) {
      cleanup();
    }
  };
}

// examples/oweeme-shop/src/islands/taskList.island.ts
var TaskItem = (el, props, ctx) => {
  const li = document.createElement("li");
  li.style.textDecoration = props.task.done ? "line-through" : "none";
  const label = document.createElement("label");
  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.checked = props.task.done;
  const onChange = () => props.onToggle(props.task.id);
  checkbox.addEventListener("change", onChange);
  ctx.onCleanup(() => checkbox.removeEventListener("change", onChange));
  label.append(checkbox, ` ${props.task.title}`);
  li.appendChild(label);
  el.appendChild(li);
};
var TaskList = (el, props, ctx) => {
  const tasks = state(props.tasks.map((t) => ({ ...t })));
  const heading = document.createElement("h2");
  heading.className = "nx-card-title";
  const list = document.createElement("ul");
  el.append(heading, list);
  const toggle = (id) => {
    tasks.value = tasks.value.map((t) => t.id === id ? { ...t, done: !t.done } : t);
  };
  let disposeItems = [];
  const disposeEffect = effect(() => {
    for (const dispose of disposeItems.splice(0)) dispose();
    const pending2 = tasks.value.filter((t) => !t.done).length;
    heading.textContent = `Pendientes: ${pending2}`;
    list.innerHTML = "";
    for (const task of tasks.value) {
      disposeItems.push(mountComponent(TaskItem, list, { task, onToggle: toggle }));
    }
  });
  ctx.onCleanup(() => {
    disposeEffect();
    for (const dispose of disposeItems.splice(0)) dispose();
  });
};
function isTask(value) {
  return !!value && typeof value === "object" && typeof value.id === "number" && typeof value.title === "string" && typeof value.done === "boolean";
}
function mount(el, props) {
  const tasks = Array.isArray(props.tasks) ? props.tasks.filter(isTask) : [];
  return mountComponent(TaskList, el, { tasks });
}
export {
  mount as default
};
