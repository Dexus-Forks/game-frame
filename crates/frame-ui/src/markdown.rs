//! Markdown to HTML for help pages, as the HTML page renders it: raw HTML in the source is
//! shown as text, never run, and `::: tip|info|warning|danger [title]` blocks become boxes.

use pulldown_cmark::{html, Event, Options, Parser};

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn plain(md: &str) -> String {
    let parser = Parser::new_ext(md, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH).map(|e| match e {
        Event::Html(t) | Event::InlineHtml(t) => Event::Text(t),
        e => e,
    });
    let mut out = String::new();
    html::push_html(&mut out, parser);
    // A link with a script scheme goes nowhere.
    out.replace("href=\"javascript:", "href=\"#")
}

/// The page as HTML, safe to put in the frame with `dangerous_inner_html`.
pub fn to_html(md: &str) -> String {
    let mut out = String::new();
    let mut chunk = String::new();
    let mut lines = md.lines();
    while let Some(line) = lines.next() {
        let Some(rest) = line.strip_prefix(":::") else {
            chunk.push_str(line);
            chunk.push('\n');
            continue;
        };
        let rest = rest.trim();
        if rest.is_empty() {
            continue;
        }
        out.push_str(&plain(&std::mem::take(&mut chunk)));
        let (kind, title) = rest.split_once(' ').unwrap_or((rest, ""));
        let body: Vec<&str> = lines.by_ref().take_while(|l| l.trim() != ":::").collect();
        if kind == "panel" {
            out.push_str(&format!("<p class=\"lib-row-meta\">The panel {} shows on the HTML page.</p>", escape(title)));
            continue;
        }
        out.push_str(&format!("<div class=\"lib-callout is-{}\">", escape(kind)));
        if !title.is_empty() {
            out.push_str(&format!("<p class=\"lib-callout-title\">{}</p>", escape(title)));
        }
        out.push_str(&plain(&body.join("\n")));
        out.push_str("</div>");
    }
    out.push_str(&plain(&chunk));
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn html_is_shown_not_run() {
        let html = super::to_html("Hi <script>x()</script> [a](javascript:alert(1))\n\n::: tip Saves\nKept **here**.\n:::\n");
        assert!(!html.contains("<script>") && html.contains("&lt;script&gt;"));
        assert!(!html.contains("javascript:"));
        assert!(html.contains("<div class=\"lib-callout is-tip\"><p class=\"lib-callout-title\">Saves</p><p>Kept <strong>here</strong>.</p>"));
    }
}
