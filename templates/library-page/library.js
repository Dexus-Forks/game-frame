// The library page (GameFrame templates/library-page): the player's game files, saves and the
// sections the game opts into, kept in this browser. It reads library.json (the data-config
// of #library) and needs no edits: the game changes library.json, the tokens and its own CSS.
// The storage is in library-store.js; the look is in tokens.css and library.css.
//
// library.json sets the words, the pictures ("logo", "backdrop"), the tokens ("theme": any
// token name without --lib-), "scheme" (dark, light, auto), "preset" (a ready-made theme in
// themes/, by name, listed in README.md), the icons ("icons": a name to a
// picture URL), each section's "icon" (a built-in icon name or a picture URL), and "panels"
// (content around the files, drawn by library-panels.js), and "pages" (Markdown pages with a
// nav bar and hash routes, drawn by library-pages.js).
//
// window.gameframeLibrary: config, ready (a promise), list(section), read(section, name),
// add(section, file), show(), hide(), icon(name). The game may set describe(section, name,
// bytes) and accept(section, name, bytes) on it (register_hooks in the Rust crate). Start sends
// the event `gameframe-library:start` to window and hides the page.
(() => {
  const root = document.getElementById('library');
  const api = (window.gameframeLibrary = window.gameframeLibrary || {});
  let config, store, tab = null;

  // The built-in icons: 24-unit SVG paths, drawn in currentColor.
  const ICONS = {
    add: 'M12 5v14M5 12h14',
    download: 'M12 4v11m0 0-4-4m4 4 4-4M5 19h14',
    delete: 'M6 6l12 12M18 6 6 18',
    up: 'M12 19V5m0 0-6 6m6-6 6 6',
    down: 'M12 5v14m0 0-6-6m6 6 6-6',
    start: 'M8 5v14l11-7z',
    file: 'M7 3h7l5 5v13H7zM14 3v5h5',
    missing: 'M12 8v5m0 3v.5M10.3 4.2 2.8 18a2 2 0 0 0 1.7 3h15a2 2 0 0 0 1.7-3L13.7 4.2a2 2 0 0 0-3.4 0z',
    game: 'M6 9h4M8 7v4M15 10h.01M18 8h.01M7 5h10a5 5 0 0 1 5 5v1a5 5 0 0 1-9 3h-2a5 5 0 0 1-9-3v-1a5 5 0 0 1 5-5z',
    saves: 'M5 3h11l3 3v15H5zM8 3v5h7V3M8 21v-7h8v7',
    mods: 'M12 3 3 8l9 5 9-5zM3 13l9 5 9-5M3 17.5l9 4.5 9-4.5',
    screenshots: 'M3 7h4l2-3h6l2 3h4v13H3zM12 17a4 4 0 1 0 0-8 4 4 0 0 0 0 8z',
    recordings: 'M3 6h13v12H3zM16 10l5-3v10l-5-3',
  };
  const el = (tag, cls, text) => { const e = document.createElement(tag); if (cls) e.className = cls; if (text != null) e.textContent = text; return e; };
  function icon(name) {
    const span = el('span', 'lib-icon'); span.setAttribute('aria-hidden', 'true');
    const url = (config.icons || {})[name] || (/[./]/.test(name) ? name : null);
    if (url) { span.classList.add('is-mask'); span.style.setProperty('--lib-icon-url', `url("${encodeURI(url)}")`); return span; }
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg'); svg.setAttribute('viewBox', '0 0 24 24');
    const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    path.setAttribute('d', ICONS[name] || ICONS.file);
    const filled = name === 'start';
    path.setAttribute('fill', filled ? 'currentColor' : 'none'); path.setAttribute('stroke', filled ? 'none' : 'currentColor');
    path.setAttribute('stroke-width', '2'); path.setAttribute('stroke-linecap', 'round'); path.setAttribute('stroke-linejoin', 'round');
    svg.append(path); span.append(svg); return span;
  }
  // Text with `backticks` as code, built from nodes, never from HTML.
  const rich = (tag, text, cls) => { const e = el(tag, cls); String(text).split('`').forEach((part, i) => e.append(i % 2 ? el('code', null, part) : part)); return e; };
  const size = (n) => n >= 1e6 ? (n / 1e6).toFixed(1) + ' MB' : Math.max(1, Math.round(n / 1e3)) + ' KB';
  const sectionIcon = (s) => s.icon || ({ 'game-data': 'game' })[s.kind] || (ICONS[s.kind] ? s.kind : 'file');
  function button(label, iconName, variant, act) {
    const b = el('button', `lib-button lib-button--${variant}`); b.type = 'button';
    if (iconName) b.append(icon(iconName));
    if (variant === 'icon') { b.title = label; b.setAttribute('aria-label', label); } else b.append(label);
    b.onclick = act; return b;
  }
  function status(text, kind) {
    const s = root.querySelector('.lib-status'); s.textContent = text || '';
    s.classList.toggle('is-bad', kind === 'bad'); s.classList.toggle('is-good', kind === 'good');
  }

  async function addFiles(id, files) {
    let n = 0;
    for (const f of files) {
      const target = id || (config.sections.find((s) => store.rules(s).import && (s.accept || []).length && store.accepts(s, f.name)) || {}).id;
      if (!target) { status(`Nothing here takes ${f.name}.`, 'bad'); continue; }
      try { await store.add(target, f); n++; } catch (e) { status(e.message, 'bad'); return render(); }
    }
    if (n) status(n === 1 ? 'Kept 1 file.' : `Kept ${n} files.`, 'good');
    render();
  }
  function picker(s) {
    const input = el('input', 'lib-hidden'); input.type = 'file'; input.multiple = store.rules(s).many; input.id = 'lib-add-' + s.id;
    if ((s.accept || []).length) input.accept = s.accept.join(',');
    input.onchange = () => addFiles(s.id, [...input.files]);
    const data = s.kind === 'game-data';
    const label = el('label', `lib-button lib-button--${data ? 'primary' : 'quiet'}`);
    label.htmlFor = input.id; label.append(icon('add'), data ? 'Choose files' : `Add ${s.label.toLowerCase()}`);
    const actions = el('div', 'lib-actions'); actions.append(label, input); return actions;
  }
  async function rows(s) {
    const r = store.rules(s), ul = el('ul', 'lib-rows'), items = await store.list(s.id);
    if (!items.length) ul.append(el('li', 'lib-row is-empty', s.empty || (s.kind === 'game-data' ? 'No game files are kept yet.' : `No ${s.label.toLowerCase()} yet.`)));
    items.forEach((e, i) => {
      const li = el('li', 'lib-row' + (e.on ? '' : ' is-off'));
      if (r.toggles) {
        const c = el('input'); c.type = 'checkbox'; c.checked = e.on; c.setAttribute('aria-label', `Use ${e.name}`);
        c.onchange = async () => { const st = await store.state(s.id); st.off = st.off.filter((n) => n !== e.name); if (!c.checked) st.off.push(e.name); await store.setState(s.id, st); render(); };
        li.append(c);
      }
      if (e.card && e.card.thumbnail) { const img = el('img', 'lib-thumb'); img.src = e.card.thumbnail; img.alt = ''; li.append(img); }
      const text = el('div', 'lib-row-text');
      const meta = [e.card && e.card.detail, s.kind === 'game-data' ? null : new Date(e.modified).toLocaleString(), size(e.size)].filter(Boolean).join(' · ');
      text.append(el('span', 'lib-row-name', (e.card && e.card.title) || e.name), el('span', 'lib-row-meta', meta));
      li.append(text);
      const move = async (to) => { const names = items.map((x) => x.name).filter((n) => n !== e.name); names.splice(to, 0, e.name); const st = await store.state(s.id); st.order = names; await store.setState(s.id, st); render(); };
      if (r.ordered) {
        const up = button('Load earlier', 'up', 'icon', () => move(i - 1)); up.disabled = i === 0;
        const down = button('Load later', 'down', 'icon', () => move(i + 1)); down.disabled = i === items.length - 1;
        li.append(up, down);
      }
      if (r.download) li.append(button(`Download ${e.name}`, 'download', 'icon', async () => {
        const url = URL.createObjectURL(new Blob([await store.read(s.id, e.name)])); const a = el('a'); a.href = url; a.download = e.name; a.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
      }));
      const del = button(`Delete ${e.name}`, 'delete', 'icon', async () => { if (confirm(`Delete ${e.name} from this browser?`)) { await store.remove(s.id, e.name); render(); } });
      del.classList.add('lib-button--danger'); li.append(del);
      ul.append(li);
    });
    return ul;
  }
  async function section(s, withSteps) {
    const box = el('section', 'lib-section'); box.dataset.section = s.id;
    if (withSteps !== null) { const h = el('h2'); h.append(icon(sectionIcon(s)), s.label); box.append(h); }
    if (withSteps && (s.steps || []).length) { const ol = el('ol', 'lib-steps'); s.steps.forEach((t) => { const li = el('li'); li.append(rich('span', t)); ol.append(li); }); box.append(ol); }
    if (withSteps) {
      const need = el('ul', 'lib-notice');
      (await store.missing()).forEach((q) => { const li = el('li'); li.append(icon('missing'), rich('span', `Missing \`${q.name}\`` + (q.hint ? `. ${q.hint}` : ''))); need.append(li); });
      box.append(need);
    }
    (s.help || []).forEach((t) => box.append(rich('p', t)));
    box.append(await rows(s));
    if (store.rules(s).import) box.append(picker(s));
    return box;
  }

  // A panel from library-panels.js. Each is drawn once and kept, so a video keeps playing.
  const drawn = {};
  async function panelBox(p, heading) {
    if (!drawn[p.id]) {
      const box = el('section', 'lib-section lib-panel'); box.dataset.panel = p.type;
      if (heading && p.title) { const h = el('h2'); h.append(icon(p.icon || 'file'), p.title); box.append(h); }
      drawn[p.id] = box;
      if (api.renderPanel) await api.renderPanel(p, box); else box.append(el('p', null, 'Add library-panels.js to show this panel.'));
    }
    return drawn[p.id];
  }

  async function render() {
    const data = config.sections.find((s) => s.kind === 'game-data'), lacking = (await store.missing()).length;
    const body = el('div');
    const panels = (config.panels || []).map((p, i) => ({ ...p, id: 'panel-' + i, label: p.title || p.type, panel: true }));
    for (const p of panels.filter((x) => x.place === 'top')) body.append(await panelBox(p, true));
    body.append(await section(data, lacking > 0));
    // The other sections and the tab panels, one tab each.
    const rest = [...config.sections.filter((s) => s !== data), ...panels.filter((p) => (p.place || 'tab') === 'tab')];
    if (rest.length) {
      if (!rest.some((s) => s.id === tab)) tab = rest[0].id;
      const tabs = el('div', 'lib-tabs'); tabs.setAttribute('role', 'tablist');
      rest.forEach((s) => {
        const b = el('button'); b.type = 'button'; b.setAttribute('role', 'tab'); b.setAttribute('aria-selected', String(s.id === tab));
        b.append(icon(s.panel ? (s.icon || 'file') : sectionIcon(s)), s.label); b.onclick = () => { tab = s.id; render(); }; tabs.append(b);
      });
      const shown = rest.find((x) => x.id === tab);
      body.append(tabs, shown.panel ? await panelBox(shown, false) : await section(shown, null));
    }
    root.querySelector('.lib-body').replaceChildren(body);
    const start = root.querySelector('.lib-start');
    start.disabled = lacking > 0;
    start.title = lacking ? 'Add the missing game files first' : '';
  }

  function frame() {
    root.dataset.scheme = config.scheme || 'dark';
    if (/^[a-z0-9-]+$/.test(config.preset || '')) {
      const link = document.createElement('link');
      link.rel = 'stylesheet';
      link.href = `themes/${config.preset}.css`;
      const components = document.querySelector('link[href$="library.css"]');
      if (components) components.after(link); else document.head.append(link);
      root.dataset.preset = config.preset;
    }
    Object.entries(config.theme || {}).forEach(([k, v]) => root.style.setProperty('--lib-' + k, v));
    if (config.backdrop) root.style.setProperty('--lib-backdrop', `url("${encodeURI(config.backdrop)}")`);
    const card = el('main', 'lib-card'), header = el('header', 'lib-header');
    if (config.logo) { const img = el('img', 'lib-logo'); img.src = config.logo; img.alt = config.title; header.append(img); }
    else header.append(el('h1', 'lib-title', config.title));
    if (config.tagline) header.append(el('p', 'lib-tagline', config.tagline));
    const start = button(config.start || 'Start', 'start', 'primary', () => {
      api.hide(); window.dispatchEvent(new CustomEvent('gameframe-library:start', { detail: { id: config.id } }));
    });
    start.classList.add('lib-start');
    const line = el('div', 'lib-status'); line.setAttribute('role', 'status'); line.setAttribute('aria-live', 'polite');
    // The library itself is the home view; library-pages.js adds the nav and the other pages.
    const home = el('div', 'lib-view lib-home'); home.append(el('div', 'lib-body'), start, line);
    card.append(header, home);
    if ((config.links || []).length) {
      const nav = el('nav', 'lib-links');
      config.links.forEach((l) => { const a = el('a', null, l.text); a.href = l.href; a.target = '_blank'; a.rel = 'noopener'; nav.append(a); });
      card.append(nav);
    }
    if (config.fine) card.append(rich('p', config.fine, 'lib-fine'));
    root.replaceChildren(el('div', 'lib-backdrop'), card);
    // Files dropped anywhere go to the first section that takes them.
    addEventListener('dragover', (e) => { if (!root.hidden) e.preventDefault(); });
    addEventListener('drop', (e) => { if (root.hidden) return; e.preventDefault(); addFiles(null, [...e.dataTransfer.files]); });
  }

  api.ready = (async () => {
    try {
      config = await (await fetch(root.dataset.config || 'library.json', { cache: 'no-cache' })).json();
      api.config = config;
      store = await window.libraryStore(config, api);
      frame();
      await render();
      if (api.startPages) api.startPages(config, root.querySelector('.lib-card'));
    } catch (e) {
      root.textContent = `The library could not start: ${e.message}`;
      throw e;
    }
  })();
  Object.assign(api, {
    list: async (id) => { await api.ready; return store.list(id); },
    read: async (id, name) => { await api.ready; return store.read(id, name); },
    add: async (id, file) => { await api.ready; await store.add(id, file); render(); },
    show: () => { root.hidden = false; render(); },
    hide: () => { root.hidden = true; },
    icon: (name) => icon(name),
  });
})();
