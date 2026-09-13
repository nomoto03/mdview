use std::path::Path;

use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag};

use crate::links;

// GFM bare-URL autolinking is not supported by pulldown-cmark;
// CommonMark angle-bracket autolinks (<https://…>) still work.
//
// `base` is the directory the document lives in. Link and image destinations
// are resolved against it here rather than being left to the WebView, which
// would normalise `../assets/x.md` and `./assets/x.md` to the same URL and
// leave no way to tell them apart.
// See docs/adr/0003-clicks-in-js-navigation-denied-in-rust.md
//
// It is `None` for text mdview produced itself, which has no directory to
// resolve against and no destinations worth rewriting.
pub fn to_html(source: &str, base: Option<&Path>) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(source, opts).map(|event| rewrite(event, base));
    let mut out = String::with_capacity(source.len() * 3 / 2);
    html::push_html(&mut out, parser);
    // The renderer takes no extra attributes and always writes images as
    // `<img src="`, so lazy loading goes on afterwards instead of by
    // reimplementing image rendering. Without it every image in a long document
    // is fetched at once, which is the cost the `mdimg` protocol exists to avoid.
    out.replace("<img src=\"", "<img loading=\"lazy\" src=\"")
}

/// Points local link and image destinations at somewhere mdview can act on.
/// Remote URLs, `mailto:`, and fragments pass through exactly as written.
fn rewrite<'a>(event: Event<'a>, base: Option<&Path>) -> Event<'a> {
    let Some(base) = base else {
        return event;
    };
    match event {
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            let dest_url = match links::resolve(&dest_url, base) {
                // A scheme the WebView cannot follow on its own. The click
                // handler recognises it; the navigation handler refuses it if
                // the click handler ever fails to run.
                Some(path) => CowStr::from(format!(
                    "mdview-open:{}",
                    links::percent_encode(&path.to_string_lossy())
                )),
                None => dest_url,
            };
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            })
        }
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            let dest_url = match links::resolve(&dest_url, base) {
                Some(path) => CowStr::from(format!(
                    "http://mdimg.localhost/{}",
                    links::percent_encode(&path.to_string_lossy())
                )),
                None => dest_url,
            };
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            })
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::to_html;
    use std::path::PathBuf;

    fn base() -> PathBuf {
        PathBuf::from(r"C:\work\docs")
    }

    fn render(source: &str) -> String {
        to_html(source, Some(base().as_path()))
    }

    #[test]
    fn table_renders() {
        let html = to_html("|a|b|\n|-|-|\n|1|2|", None);
        assert!(html.contains("<table>"), "output: {html}");
        assert!(html.contains("<th>a</th>"), "output: {html}");
        assert!(html.contains("<td>1</td>"), "output: {html}");
    }

    #[test]
    fn task_list_renders_checkboxes() {
        let html = to_html("- [x] done\n- [ ] todo", None);
        assert_eq!(html.matches("type=\"checkbox\"").count(), 2, "output: {html}");
        assert_eq!(html.matches("checked").count(), 1, "output: {html}");
        assert!(html.contains("disabled"), "output: {html}");
    }

    #[test]
    fn code_block_plain() {
        let html = to_html("```rust\nfn main() {}\n```", None);
        assert!(html.contains("<pre><code"), "output: {html}");
        assert!(!html.contains("<span class="), "output: {html}");
    }

    #[test]
    fn strikethrough_renders() {
        let html = to_html("~~gone~~", None);
        assert!(html.contains("<del>gone</del>"), "output: {html}");
    }

    #[test]
    fn japanese_passthrough() {
        let html = to_html("# 見出し\n\n日本語の段落です。", None);
        assert!(html.contains("<h1>見出し</h1>"), "output: {html}");
        assert!(html.contains("日本語の段落です。"), "output: {html}");
    }

    // Raw HTML passes through untouched: input files are trusted local
    // documents and script execution is blocked by the WebView CSP instead.
    #[test]
    fn raw_html_passthrough() {
        let html = to_html("<b>bold</b>", None);
        assert!(html.contains("<b>bold</b>"), "output: {html}");
    }

    // No headings carry ids, so a table-of-contents link has nowhere to land.
    // It stays a fragment rather than being mistaken for a file.
    #[test]
    fn fragment_links_are_untouched() {
        let html = render("[目次](#section)");
        assert!(html.contains("href=\"#section\""), "output: {html}");
    }

    #[test]
    fn remote_links_are_untouched() {
        let html = render("[GitHub](https://github.com/)");
        assert!(html.contains("href=\"https://github.com/\""), "output: {html}");
    }

    #[test]
    fn local_links_become_a_scheme_the_webview_cannot_follow() {
        let html = render("[別資料](../index.md)");
        assert!(html.contains("href=\"mdview-open:"), "output: {html}");
        // C%3A%5Cwork%5Cindex.md
        assert!(html.contains("C%3A%5Cwork%5Cindex.md"), "output: {html}");
    }

    #[test]
    fn local_images_are_served_over_the_private_protocol() {
        let html = render("![図](./assets/logo.png)");
        assert!(html.contains("http://mdimg.localhost/"), "output: {html}");
        assert!(html.contains("C%3A%5Cwork%5Cdocs%5Cassets%5Clogo.png"), "output: {html}");
    }

    #[test]
    fn images_are_lazily_loaded() {
        let html = render("![図](./a.png)");
        assert!(html.contains("<img loading=\"lazy\" src=\""), "output: {html}");
    }

    #[test]
    fn remote_images_are_untouched() {
        let html = render("![x](https://example.com/a.png)");
        assert!(html.contains("src=\"https://example.com/a.png\""), "output: {html}");
    }

    // Reading a network path hands the user's NTLM hash to whatever host the
    // document named, so it is never rewritten into something mdview will fetch.
    //
    // Percent-escapes are the form that matters. Written literally, CommonMark
    // reads the leading `\\` as an escaped backslash and the destination
    // arrives with only one, naming an ordinary local file rather than a host;
    // escaping is what carries a genuine network path through the parser.
    #[test]
    fn network_images_are_not_rewritten() {
        let html = render("![x](%5C%5Cevil%5Cshare%5Ca.png)");
        assert!(!html.contains("mdimg.localhost"), "output: {html}");
    }

    // Without a base directory there is nothing to resolve against, so mdview's
    // own usage text cannot accidentally produce local destinations.
    #[test]
    fn without_a_base_nothing_is_rewritten() {
        let html = to_html("[x](./other.md)", None);
        assert!(html.contains("href=\"./other.md\""), "output: {html}");
    }
}
