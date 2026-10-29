// The library page's other pages (GameFrame templates/library-page): help, guides, news, as
// Markdown files, with a nav bar and hash routes (#/help), so they work on any static host with
// no server set-up. library.json "pages": [{"path": "help", "title": "Help", "src": "help.md",
// "icon": "file"}]. A page takes "text" instead of "src", and the Markdown can hold ::: tip
// boxes and ::: panel NAME slots (library-panels.js). The library itself is the home page, #/.
// A link in Markdown to "#/help" opens that page.
(() => {
  const api = (window.gameframeLibrary = window.gameframeLibrary || {});
  const el = (tag, cls, text) => { const e = document.createElement(tag); if (cls) e.className = cls; if (text != null) e.textContent = text; return e; };

  api.startPages = (config, card) => {
    const pages = config.pages || [];
    if (!pages.length) return;
    const home = card.querySelector('.lib-home'), view = el('div', 'lib-view lib-page');
    const nav = el('nav', 'lib-nav'); nav.setAttribute('aria-label', 'Pages');
    const links = [{ path: '', title: config.home || 'Library', icon: 'game' }, ...pages].map((p) => {
      const a = el('a'); a.href = '#/' + p.path; a.dataset.path = p.path;
      if (api.icon) a.append(api.icon(p.icon || 'file'));
      a.append(p.title); nav.append(a); return a;
    });
    card.querySelector('.lib-header').after(nav);
    home.after(view);
    const drawn = {};

    async function show() {
      const path = decodeURIComponent(location.hash.replace(/^#\/?/, ''));
      const page = pages.find((p) => p.path === path);
      links.forEach((a) => a.toggleAttribute('aria-current', a.dataset.path === (page ? page.path : '')));
      home.hidden = !!page; view.hidden = !page;
      if (!page) return;
      if (!drawn[page.path]) {
        const box = el('article', 'lib-section'); drawn[page.path] = box;
        if (page.title) box.append(el('h2', null, page.title));
        try {
          const text = page.text ?? await (await fetch(page.src, { cache: 'no-cache' })).text();
          const body = api.markdown(text);
          box.append(body);
          await api.fillEmbeds(body, config);
        } catch (e) {
          box.append(el('p', 'lib-status is-bad', `This page could not load: ${e.message}`));
        }
      }
      view.replaceChildren(drawn[page.path]);
      card.scrollIntoView({ block: 'start' });
    }
    addEventListener('hashchange', show);
    show();
  };
})();
