// The library page's panels (GameFrame templates/library-page): content around the files, set
// in library.json "panels". Each panel has a "type", a "title", and "place": "tab" (the
// default, a tab beside the sections) or "top" (above the game files, for a notice). A "src"
// is a file next to the page (static, shipped with it) or any URL (fetched when the page opens).
//
//   markdown   {"src": "news.md"} or "text"             Markdown, rendered here
//   changelog  {"src": "CHANGELOG.md", "limit": 3}      the newest "## " releases of a changelog
//   messages   {"src": "messages.json"} or "items"      [{date, title, text, level}] notices
//   youtube    {"id": "VIDEO_ID"}                       loads from YouTube only when clicked
//   releases   {"repo": "owner/name", "limit": 3}       GitHub releases, fetched live
//
// A panel with a "name" can be placed inside Markdown with `::: panel NAME` and `:::`; set
// "place": "none" to show it only there. Markdown also takes ::: tip, info, warning and danger.
// A game adds its own type: gameframeLibrary.panels.myType = async (panel, el, lib) => node.
// The Markdown is built as page nodes, never as HTML, so a file cannot run script in the page.
(() => {
  const api = (window.gameframeLibrary = window.gameframeLibrary || {});
  const el = (tag, cls, text) => { const e = document.createElement(tag); if (cls) e.className = cls; if (text != null) e.textContent = text; return e; };
  const safe = (href) => /^(https?:|mailto:|#|\/|\.{0,2}\/|[\w-]+\.)/i.test(href) && !/^\s*javascript:/i.test(href) ? href : '#';
  const text = async (src) => { const r = await fetch(src, { cache: 'no-cache' }); if (!r.ok) throw new Error(`${src}: ${r.status}`); return r.text(); };

  // Inline Markdown: `code`, **strong**, *em*, [links](url), ![pictures](url).
  function inline(parent, s) {
    const re = /(`[^`]+`)|(\*\*[^*]+\*\*)|(\*[^*]+\*|_[^_]+_)|(!?\[[^\]]*\]\([^)\s]+\))/;
    let m;
    while ((m = re.exec(s))) {
      if (m.index) parent.append(s.slice(0, m.index));
      const t = m[0];
      if (m[1]) parent.append(el('code', null, t.slice(1, -1)));
      else if (m[2]) inline(parent.appendChild(el('strong')), t.slice(2, -2));
      else if (m[3]) inline(parent.appendChild(el('em')), t.slice(1, -1));
      else {
        const [, label, href] = /!?\[([^\]]*)\]\(([^)\s]+)\)/.exec(t);
        if (t[0] === '!') { const img = el('img', 'lib-md-img'); img.src = safe(href); img.alt = label; img.loading = 'lazy'; parent.append(img); }
        else { const a = el('a'); a.href = safe(href); a.target = '_blank'; a.rel = 'noopener'; inline(a, label); parent.append(a); }
      }
      s = s.slice(m.index + t.length);
    }
    if (s) parent.append(s);
    return parent;
  }
  // Block Markdown: headings, paragraphs, lists, quotes, fenced code, rules.
  function markdown(src, headingShift = 0) {
    const out = el('div', 'lib-md'), lines = src.replace(/\r/g, '').split('\n');
    let i = 0;
    while (i < lines.length) {
      const line = lines[i];
      if (!line.trim()) { i++; continue; }
      let m;
      if ((m = /^:::\s*(\w+)\s*(.*)$/.exec(line))) {
        // A container, as VitePress writes them: ::: tip|info|warning|danger [title], or
        // ::: panel NAME, which places the panel named NAME from library.json there.
        const body = []; i++;
        let depth = 1;
        while (i < lines.length) {
          if (/^:::\s*\w/.test(lines[i])) depth++;
          else if (/^:::\s*$/.test(lines[i]) && --depth === 0) break;
          body.push(lines[i++]);
        }
        i++;
        if (m[1] === 'panel') { const slot = el('div', 'lib-embed'); slot.dataset.panel = m[2].trim(); out.append(slot); }
        else {
          const box = el('div', `lib-callout is-${m[1]}`);
          if (m[2]) box.append(inline(el('p', 'lib-callout-title'), m[2]));
          box.append(markdown(body.join('\n'), headingShift)); out.append(box);
        }
      } else if (line.startsWith('```')) {
        const code = []; i++;
        while (i < lines.length && !lines[i].startsWith('```')) code.push(lines[i++]);
        i++; const pre = el('pre'); pre.append(el('code', null, code.join('\n'))); out.append(pre);
      } else if ((m = /^(#{1,6})\s+(.*)/.exec(line))) {
        out.append(inline(el('h' + Math.min(6, m[1].length + 2 + headingShift)), m[2])); i++;
      } else if (/^(-{3,}|\*{3,})\s*$/.test(line)) { out.append(el('hr')); i++; }
      else if (/^\s*([-*+]|\d+\.)\s+/.test(line)) {
        const ordered = /^\s*\d+\./.test(line), list = el(ordered ? 'ol' : 'ul');
        while (i < lines.length && /^\s*([-*+]|\d+\.)\s+/.test(lines[i])) {
          let item = lines[i++].replace(/^\s*([-*+]|\d+\.)\s+/, '');
          while (i < lines.length && /^\s{2,}\S/.test(lines[i]) && !/^\s*([-*+]|\d+\.)\s+/.test(lines[i])) item += ' ' + lines[i++].trim();
          list.append(inline(el('li'), item));
        }
        out.append(list);
      } else if (line.startsWith('>')) {
        const q = [];
        while (i < lines.length && lines[i].startsWith('>')) q.push(lines[i++].replace(/^>\s?/, ''));
        const b = el('blockquote'); b.append(markdown(q.join('\n'), headingShift)); out.append(b);
      } else {
        const p = [];
        while (i < lines.length && lines[i].trim() && !/^(#{1,6}\s|```|>|\s*([-*+]|\d+\.)\s)/.test(lines[i])) p.push(lines[i++].trim());
        out.append(inline(el('p'), p.join(' ')));
      }
    }
    return out;
  }

  const panels = (api.panels = api.panels || {});
  panels.markdown ||= async (p) => markdown(p.text ?? await text(p.src));
  panels.changelog ||= async (p) => {
    const md = await text(p.src || 'CHANGELOG.md');
    const parts = md.split(/\n(?=## )/).filter((s) => s.startsWith('## '));
    return markdown(parts.slice(0, p.limit || 3).join('\n\n'));
  };
  panels.messages ||= async (p) => {
    const items = p.items || JSON.parse(await text(p.src));
    const ul = el('ul', 'lib-messages');
    if (!items.length) ul.append(el('li', 'lib-row is-empty', 'No messages.'));
    items.forEach((m) => {
      const li = el('li', 'lib-message' + (m.level ? ' is-' + m.level : ''));
      const head = el('div', 'lib-message-head');
      if (m.title) head.append(el('strong', null, m.title));
      if (m.date) head.append(el('span', 'lib-row-meta', m.date));
      li.append(head);
      if (m.text) li.append(markdown(m.text));
      ul.append(li);
    });
    return ul;
  };
  panels.youtube ||= async (p) => {
    // Nothing loads from YouTube until the player asks: a button stands in for the video.
    const box = el('div', 'lib-video');
    const go = el('button', 'lib-button lib-button--quiet');
    go.type = 'button'; go.append(api.icon ? api.icon('start') : '', p.label || 'Play the video (loads from YouTube)');
    go.onclick = () => {
      const f = el('iframe'); f.src = `https://www.youtube-nocookie.com/embed/${encodeURIComponent(p.id)}?autoplay=1`;
      f.title = p.title || 'Video'; f.allow = 'autoplay; encrypted-media; picture-in-picture'; f.allowFullscreen = true;
      box.replaceChildren(f);
    };
    box.append(go);
    return box;
  };
  panels.releases ||= async (p) => {
    const r = await fetch(`https://api.github.com/repos/${p.repo}/releases?per_page=${p.limit || 3}`);
    if (!r.ok) throw new Error(`GitHub releases: ${r.status}`);
    const box = el('div');
    (await r.json()).forEach((rel) => {
      const h = el('h3'); const a = el('a', null, rel.name || rel.tag_name); a.href = rel.html_url; a.target = '_blank'; a.rel = 'noopener'; h.append(a);
      box.append(h, el('p', 'lib-row-meta', new Date(rel.published_at).toLocaleDateString()), markdown(rel.body || '', 1));
    });
    return box;
  };

  // Draw one panel into a section element; a failure shows in the panel, not the whole page.
  api.renderPanel = async (panel, into) => {
    const make = panels[panel.type];
    try {
      if (!make) throw new Error(`no panel type ${panel.type}`);
      const node = await make(panel, into, api);
      into.append(node);
      if (api.config && node.querySelector) await api.fillEmbeds(node, api.config);
    } catch (e) {
      into.append(el('p', 'lib-status is-bad', `This panel could not load: ${e.message}`));
    }
  };
  // Fill the ::: panel slots of rendered Markdown with the panels named in library.json.
  api.fillEmbeds = async (node, config) => {
    for (const slot of node.querySelectorAll('.lib-embed')) {
      const panel = (config.panels || []).find((p) => p.name === slot.dataset.panel);
      if (panel) await api.renderPanel(panel, slot);
      else slot.append(el('p', 'lib-status is-bad', `No panel named ${slot.dataset.panel}.`));
    }
  };
  api.markdown = markdown;
})();
