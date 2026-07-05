use pulldown_cmark::{html, Options, Parser};

// GFM bare-URL autolinking is not supported by pulldown-cmark;
// CommonMark angle-bracket autolinks (<https://…>) still work.
pub fn to_html(source: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(source, opts);
    let mut out = String::with_capacity(source.len() * 3 / 2);
    html::push_html(&mut out, parser);
    out
}

#[cfg(test)]
mod tests {
    use super::to_html;

    #[test]
    fn table_renders() {
        let html = to_html("|a|b|\n|-|-|\n|1|2|");
        assert!(html.contains("<table>"), "output: {html}");
        assert!(html.contains("<th>a</th>"), "output: {html}");
        assert!(html.contains("<td>1</td>"), "output: {html}");
    }

    #[test]
    fn task_list_renders_checkboxes() {
        let html = to_html("- [x] done\n- [ ] todo");
        assert_eq!(html.matches("type=\"checkbox\"").count(), 2, "output: {html}");
        assert_eq!(html.matches("checked").count(), 1, "output: {html}");
        assert!(html.contains("disabled"), "output: {html}");
    }

    #[test]
    fn code_block_plain() {
        let html = to_html("```rust\nfn main() {}\n```");
        assert!(html.contains("<pre><code"), "output: {html}");
        assert!(!html.contains("<span class="), "output: {html}");
    }

    #[test]
    fn strikethrough_renders() {
        let html = to_html("~~gone~~");
        assert!(html.contains("<del>gone</del>"), "output: {html}");
    }

    #[test]
    fn japanese_passthrough() {
        let html = to_html("# 見出し\n\n日本語の段落です。");
        assert!(html.contains("<h1>見出し</h1>"), "output: {html}");
        assert!(html.contains("日本語の段落です。"), "output: {html}");
    }

    // Raw HTML passes through untouched: input files are trusted local
    // documents and script execution is blocked by the WebView CSP instead.
    #[test]
    fn raw_html_passthrough() {
        let html = to_html("<b>bold</b>");
        assert!(html.contains("<b>bold</b>"), "output: {html}");
    }
}
