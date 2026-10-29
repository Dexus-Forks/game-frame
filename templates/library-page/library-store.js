// The library page's storage (GameFrame templates/library-page): IndexedDB in the layout the
// Rust crate gameframe-library uses (src/web.rs), so the page and the game read the same files.
//
// Database: library.json's "id", version 1. Object stores:
//   files    the bytes of each file (Uint8Array), key SECTION/NAME
//   entries  JSON text of {name, size, modified, card}, key SECTION/NAME
//   state    JSON text of {order, off}, key SECTION; the encryption key under "#key"
// With "encrypt" (the default), each file is AES-GCM: 12 bytes of IV, then the ciphertext. The
// key is a WebCrypto key made here or by the game, marked non-extractable: scripts can use it
// but never read it, so a copy of the files is unreadable without this browser.
//
// libraryStore(config) gives {list, read, add, remove, state, setState, rules, accepts}.
window.libraryStore = async (config, hooks) => {
  const KEY = '#key';
  const req = (r) => new Promise((ok, fail) => { r.onsuccess = () => ok(r.result); r.onerror = () => fail(r.error); });
  const done = (tx) => new Promise((ok, fail) => { tx.oncomplete = ok; tx.onerror = () => fail(tx.error); tx.onabort = () => fail(tx.error); });
  const db = await new Promise((ok, fail) => {
    const r = indexedDB.open(config.id, 1);
    r.onupgradeneeded = () => ['files', 'entries', 'state'].forEach((s) => r.result.objectStoreNames.contains(s) || r.result.createObjectStore(s));
    r.onsuccess = () => ok(r.result); r.onerror = () => fail(r.error);
  });
  const range = (id) => IDBKeyRange.bound(id + '/', id + '/￿');

  let key = null;
  if (config.encrypt !== false) {
    key = await req(db.transaction('state').objectStore('state').get(KEY));
    if (!key) {
      key = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, false, ['encrypt', 'decrypt']);
      const tx = db.transaction('state', 'readwrite'); tx.objectStore('state').put(key, KEY); await done(tx);
    }
  }
  async function seal(bytes) {
    if (!key) return bytes;
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const out = new Uint8Array(12 + bytes.length + 16);
    out.set(iv); out.set(new Uint8Array(await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, bytes)), 12);
    return out;
  }
  async function unseal(bytes) {
    if (!key || bytes == null) return bytes;
    return new Uint8Array(await crypto.subtle.decrypt({ name: 'AES-GCM', iv: bytes.subarray(0, 12) }, key, bytes.subarray(12)));
  }

  const on = (s, field, value) => s[field] ?? value;
  const rules = (s) => ({
    many: on(s, 'many', true), ordered: on(s, 'ordered', s.kind === 'mods'), toggles: on(s, 'toggles', s.kind === 'mods'),
    download: on(s, 'download', s.kind !== 'game-data'), import: on(s, 'import', true),
    newest: ['saves', 'screenshots', 'recordings'].includes(s.kind),
  });
  const accepts = (s, name) => !(s.accept || []).length || s.accept.some((e) => name.toLowerCase().endsWith(e.toLowerCase()));
  const section = (id) => config.sections.find((s) => s.id === id);

  const entries = async (id) => (await req(db.transaction('entries').objectStore('entries').getAll(range(id)))).map((t) => JSON.parse(t));
  const read = async (id, name) => unseal(await req(db.transaction('files').objectStore('files').get(id + '/' + name)));
  async function state(id) {
    const t = await req(db.transaction('state').objectStore('state').get(id));
    return Object.assign({ order: [], off: [] }, t ? JSON.parse(t) : {});
  }
  async function setState(id, s) { const tx = db.transaction('state', 'readwrite'); tx.objectStore('state').put(JSON.stringify(s), id); await done(tx); }

  // The page's order: the player's for ordered sections, newest first for saves, else by name.
  async function list(id) {
    const s = section(id), r = rules(s), st = await state(id);
    const rows = (await entries(id)).sort((a, b) => a.name.localeCompare(b.name));
    if (r.ordered) { const rank = (e) => { const i = st.order.indexOf(e.name); return i < 0 ? 1e9 : i; }; rows.sort((a, b) => rank(a) - rank(b)); }
    else if (r.newest) rows.sort((a, b) => b.modified - a.modified);
    return rows.map((e) => ({ ...e, on: !r.toggles || !st.off.includes(e.name) }));
  }

  // Keep a file: the section's checks, the game's (hooks.accept, hooks.describe), one transaction.
  async function add(id, file) {
    const s = section(id), r = rules(s);
    if (!accepts(s, file.name)) throw new Error(`${s.label} takes ${s.accept.join(', ')} files, not ${file.name}`);
    const bytes = new Uint8Array(await file.arrayBuffer());
    const why = hooks.accept && hooks.accept(id, file.name, bytes);
    if (why) throw new Error(why);
    const card = hooks.describe ? hooks.describe(id, file.name, bytes) : null;
    const before = await entries(id), sealed = await seal(bytes);
    const entry = { name: file.name, size: bytes.length, modified: Date.now() };
    if (card) entry.card = typeof card === 'string' ? JSON.parse(card) : card;
    const tx = db.transaction(['files', 'entries'], 'readwrite');
    tx.objectStore('files').put(sealed, id + '/' + file.name);
    tx.objectStore('entries').put(JSON.stringify(entry), id + '/' + file.name);
    if (!r.many) before.filter((e) => e.name !== file.name).forEach((e) => {
      tx.objectStore('files').delete(id + '/' + e.name); tx.objectStore('entries').delete(id + '/' + e.name);
    });
    await done(tx);
    if (r.ordered) { const st = await state(id); if (!st.order.includes(file.name)) { st.order.push(file.name); await setState(id, st); } }
  }

  async function remove(id, name) {
    const tx = db.transaction(['files', 'entries'], 'readwrite');
    tx.objectStore('files').delete(id + '/' + name); tx.objectStore('entries').delete(id + '/' + name);
    await done(tx);
    const st = await state(id);
    st.order = st.order.filter((n) => n !== name); st.off = st.off.filter((n) => n !== name);
    await setState(id, st);
  }

  // The required game files that are not kept, or whose size is wrong.
  async function missing() {
    const s = config.sections.find((x) => x.kind === 'game-data'), kept = await entries(s.id);
    return (s.required || []).filter((q) => {
      const e = kept.find((k) => k.name.toLowerCase() === q.name.toLowerCase());
      return !e || (q.size != null && e.size !== q.size);
    });
  }

  return { list, read, add, remove, state, setState, missing, rules, accepts, section };
};
