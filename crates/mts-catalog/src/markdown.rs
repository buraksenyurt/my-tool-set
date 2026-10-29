//! README rendering: Markdown → sanitized HTML with every URL rewritten.
//!
//! - Relative images become asset URLs served by the shell's `mtsimg`
//!   protocol, so the webview never talks to the network itself.
//! - Relative links point at the file on GitHub (release tools) or are
//!   dropped (local tools: there is nothing to link to).
//! - Absolute images are kept only when they live in the tool's own repository
//!   (rewritten to asset URLs too); anything else would be blocked by the CSP
//!   anyway, so its `src` is removed and the alt text remains.
//! - Headings get GitHub-style ids with a `user-content-` prefix, so in-page
//!   links work and README ids cannot clash with the shell's own elements.
//! - ammonia removes scripts, event handlers, styles and unknown schemes.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};

/// Characters escaped in each path segment of a generated URL.
const SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'/')
    .add(b'\\');

pub const ID_PREFIX: &str = "user-content-";

/// Where a README's relative URLs should point.
#[derive(Debug, Clone)]
pub struct DocContext {
    pub tool: String,
    /// Tag, `HEAD`, or `local`.
    pub revision: String,
    /// `owner/name` for release tools.
    pub repo: Option<String>,
    /// `mtsimg://localhost` or, on Windows, `http://mtsimg.localhost`.
    pub asset_base: String,
    /// The README's own path inside the repository or folder, e.g. `README.md`.
    pub readme_path: String,
}

impl DocContext {
    /// URL the shell serves `path` (relative to the repository root) under.
    pub fn asset_url(&self, path: &str) -> String {
        asset_url(&self.asset_base, &self.tool, &self.revision, path)
    }

    fn readme_dir(&self) -> &str {
        match self.readme_path.rsplit_once('/') {
            Some((dir, _)) => dir,
            None => "",
        }
    }
}

pub fn asset_url(base: &str, tool: &str, revision: &str, path: &str) -> String {
    format!(
        "{base}/{}/{}/{}",
        utf8_percent_encode(tool, SEGMENT),
        utf8_percent_encode(revision, SEGMENT),
        encode_path(path)
    )
}

/// Percent-encodes each `/`-separated segment of `path`.
pub fn encode_path(path: &str) -> String {
    path.split('/')
        .map(|s| utf8_percent_encode(s, SEGMENT).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn render(markdown: &str, ctx: &DocContext) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);

    let mut events: Vec<Event> = Parser::new_ext(markdown, options).collect();
    add_heading_ids(&mut events);

    let mut raw = String::with_capacity(markdown.len() * 2);
    html::push_html(&mut raw, events.into_iter());
    sanitize(&raw, ctx)
}

/// GitHub-style slugs: lowercase, punctuation removed, spaces → `-`,
/// duplicates get `-1`, `-2`, ...
fn add_heading_ids(events: &mut [Event]) {
    let mut used: HashMap<String, usize> = HashMap::new();
    let mut i = 0;
    while i < events.len() {
        if let Event::Start(Tag::Heading { id: None, .. }) = &events[i] {
            let mut text = String::new();
            let mut j = i + 1;
            while j < events.len() && !matches!(events[j], Event::End(TagEnd::Heading(_))) {
                if let Event::Text(t) | Event::Code(t) = &events[j] {
                    text.push_str(t);
                }
                j += 1;
            }
            let base = slugify(&text);
            let count = used.entry(base.clone()).or_insert(0);
            let slug = if *count == 0 {
                base
            } else {
                format!("{base}-{count}")
            };
            *count += 1;
            if let Event::Start(Tag::Heading { id, .. }) = &mut events[i] {
                *id = Some(slug.into());
            }
            i = j;
        }
        i += 1;
    }
}

pub fn slugify(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            '-' | '_' => Some(c),
            c if c.is_alphanumeric() => Some(c),
            _ => None,
        })
        .collect()
}

fn sanitize(html: &str, ctx: &DocContext) -> String {
    let ctx = ctx.clone();
    let mut builder = ammonia::Builder::default();
    builder
        .add_generic_attributes(["id"])
        .id_prefix(Some(ID_PREFIX))
        .link_rel(Some("noopener noreferrer"))
        .url_schemes(HashSet::from(["http", "https", "mailto", "mtsimg"]))
        .url_relative(ammonia::UrlRelative::PassThrough)
        .attribute_filter(
            move |element, attribute, value| match (element, attribute) {
                ("img", "src") => rewrite_image(value, &ctx).map(Cow::Owned),
                ("a", "href") => rewrite_link(value, &ctx),
                _ => Some(Cow::Borrowed(value)),
            },
        );
    builder.clean(html).to_string()
}

fn is_absolute(url: &str) -> bool {
    url.contains("://") || url.starts_with("//") || url.starts_with("mailto:")
}

fn rewrite_image(src: &str, ctx: &DocContext) -> Option<String> {
    if src.starts_with("data:") {
        return None;
    }
    if is_absolute(src) {
        // Only pictures from the tool's own repository can be served.
        let path = ctx
            .repo
            .as_deref()
            .and_then(|repo| same_repo_path(src, repo))?;
        return Some(ctx.asset_url(&path));
    }
    let path = resolve(ctx.readme_dir(), strip_query(src))?;
    Some(ctx.asset_url(&path))
}

fn rewrite_link<'u>(href: &'u str, ctx: &DocContext) -> Option<Cow<'u, str>> {
    if href.starts_with('#') || is_absolute(href) {
        return Some(Cow::Borrowed(href));
    }
    let repo = ctx.repo.as_deref()?;
    let (path, fragment) = match href.split_once('#') {
        Some((p, f)) => (p, Some(f)),
        None => (href, None),
    };
    let path = resolve(ctx.readme_dir(), strip_query(path))?;
    let mut url = format!("https://github.com/{repo}/blob/{}/{path}", ctx.revision);
    if let Some(f) = fragment {
        url.push('#');
        url.push_str(f);
    }
    Some(Cow::Owned(url))
}

fn strip_query(s: &str) -> &str {
    s.split(['?', '#']).next().unwrap_or(s)
}

/// Joins `relative` onto `dir` and normalizes `.`/`..`. `None` if the result
/// would leave the repository.
pub fn resolve(dir: &str, relative: &str) -> Option<String> {
    let relative = relative.trim_start_matches("./");
    let decoded = percent_encoding::percent_decode_str(relative)
        .decode_utf8()
        .ok()?;
    let mut parts: Vec<&str> = if relative.starts_with('/') {
        Vec::new()
    } else {
        dir.split('/').filter(|s| !s.is_empty()).collect()
    };
    for part in decoded.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            p => parts.push(p),
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

/// `https://raw.githubusercontent.com/<repo>/<rev>/<path>` or
/// `https://github.com/<repo>/(raw|blob)/<rev>/<path>` → `<path>`.
fn same_repo_path(url: &str, repo: &str) -> Option<String> {
    let lower_repo = repo.to_ascii_lowercase();
    let rest = url
        .strip_prefix("https://raw.githubusercontent.com/")
        .map(|r| (r, false))
        .or_else(|| url.strip_prefix("https://github.com/").map(|r| (r, true)))?;
    let (rest, github) = rest;
    let (owner_name, after) = split_n(rest, 2)?;
    if owner_name.to_ascii_lowercase() != lower_repo {
        return None;
    }
    let after = if github {
        after
            .strip_prefix("raw/")
            .or_else(|| after.strip_prefix("blob/"))?
    } else {
        after
    };
    // Skip the revision segment: the shell serves its own revision.
    let (_, path) = after.split_once('/')?;
    resolve("", strip_query(path))
}

fn split_n(s: &str, n: usize) -> Option<(&str, &str)> {
    let mut idx = 0;
    for _ in 0..n {
        idx += s[idx..].find('/')? + 1;
    }
    Some((&s[..idx - 1], &s[idx..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release_ctx() -> DocContext {
        DocContext {
            tool: "easy-git".into(),
            revision: "v0.1.0".into(),
            repo: Some("buraksenyurt/easy-git".into()),
            asset_base: "mtsimg://localhost".into(),
            readme_path: "README.md".into(),
        }
    }

    #[test]
    fn rewrites_relative_images_to_the_asset_protocol() {
        let html = render("![Genel](docs/images/overview.png)", &release_ctx());
        assert!(
            html.contains(r#"src="mtsimg://localhost/easy-git/v0.1.0/docs/images/overview.png""#),
            "{html}"
        );
        assert!(html.contains(r#"alt="Genel""#));
    }

    #[test]
    fn encodes_spaces_and_non_ascii_in_asset_paths() {
        let html = render("![x](<./görsel klasörü/ekran 1.png>)", &release_ctx());
        assert!(
            html.contains("g%C3%B6rsel%20klas%C3%B6r%C3%BC/ekran%201.png"),
            "{html}"
        );
    }

    #[test]
    fn relative_links_go_to_github_blob_at_the_revision() {
        let html = render("[Plan](docs/PLAN.md#faz-1)", &release_ctx());
        assert!(
            html.contains(
                r#"href="https://github.com/buraksenyurt/easy-git/blob/v0.1.0/docs/PLAN.md#faz-1""#
            ),
            "{html}"
        );
        assert!(html.contains(r#"rel="noopener noreferrer""#));
    }

    #[test]
    fn readme_in_a_subfolder_resolves_against_its_folder() {
        let mut ctx = release_ctx();
        ctx.readme_path = "docs/README.md".into();
        let html = render("![a](images/a.png) ![b](../b.png) ![c](../../c.png)", &ctx);
        assert!(html.contains("/v0.1.0/docs/images/a.png"), "{html}");
        assert!(html.contains("/v0.1.0/b.png"), "{html}");
        // ../../c.png escapes the repository: no src at all.
        assert_eq!(html.matches("src=").count(), 2, "{html}");
    }

    #[test]
    fn absolute_images_only_from_the_same_repository() {
        let html = render(
            "![a](https://raw.githubusercontent.com/buraksenyurt/easy-git/main/docs/a.png) \
             ![b](https://github.com/buraksenyurt/easy-git/blob/main/b.png?raw=true) \
             ![c](https://img.shields.io/badge/x.svg) \
             ![d](https://raw.githubusercontent.com/someone/else/main/d.png)",
            &release_ctx(),
        );
        assert!(
            html.contains("mtsimg://localhost/easy-git/v0.1.0/docs/a.png"),
            "{html}"
        );
        assert!(
            html.contains("mtsimg://localhost/easy-git/v0.1.0/b.png"),
            "{html}"
        );
        assert!(!html.contains("shields.io"), "{html}");
        assert!(!html.contains("someone"), "{html}");
    }

    #[test]
    fn strips_scripts_handlers_styles_and_javascript_urls() {
        let html = render(
            "<script>alert(1)</script>\n\n<img src=\"a.png\" onerror=\"alert(2)\" style=\"x\">\n\n\
             [x](javascript:alert(3)) <a href=\"vbscript:x\">y</a>",
            &release_ctx(),
        );
        for bad in [
            "<script",
            "alert",
            "onerror",
            "style=",
            "javascript:",
            "vbscript:",
        ] {
            assert!(!html.contains(bad), "{bad} survived: {html}");
        }
    }

    #[test]
    fn keeps_raw_html_tables_like_the_coding_vibes_readme() {
        let html = render(
            "<table><tr><td valign=\"top\"><strong>JDepend</strong></td></tr></table>",
            &release_ctx(),
        );
        assert!(
            html.contains("<table>") && html.contains("<strong>JDepend</strong>"),
            "{html}"
        );
    }

    #[test]
    fn headings_get_prefixed_github_slugs() {
        let html = render(
            "# Kurulum ve Kullanım\n\n## Kurulum ve Kullanım\n\n### `npm` & Tauri!",
            &release_ctx(),
        );
        assert!(
            html.contains(r#"id="user-content-kurulum-ve-kullanım""#),
            "{html}"
        );
        assert!(
            html.contains(r#"id="user-content-kurulum-ve-kullanım-1""#),
            "{html}"
        );
        assert!(html.contains(r#"id="user-content-npm--tauri""#), "{html}");
    }

    #[test]
    fn local_tools_drop_relative_links_but_keep_images() {
        let ctx = DocContext {
            tool: "git-dash".into(),
            revision: "local".into(),
            repo: None,
            asset_base: "http://mtsimg.localhost".into(),
            readme_path: "README.md".into(),
        };
        let html = render(
            "[öğreti](./docs/zig.md) ![r](runtime_00.png) [site](https://ziglang.org)",
            &ctx,
        );
        assert!(!html.contains("docs/zig.md"), "{html}");
        assert!(html.contains("öğreti"), "{html}");
        assert!(
            html.contains("http://mtsimg.localhost/git-dash/local/runtime_00.png"),
            "{html}"
        );
        assert!(html.contains("https://ziglang.org"), "{html}");
    }

    #[test]
    fn resolve_normalizes_and_refuses_to_escape() {
        assert_eq!(resolve("", "a/./b/../c.png").as_deref(), Some("a/c.png"));
        assert_eq!(resolve("docs", "/root.png").as_deref(), Some("root.png"));
        assert_eq!(resolve("", "../x"), None);
        assert_eq!(resolve("", "a%20b.png").as_deref(), Some("a b.png"));
        assert_eq!(resolve("", ""), None);
    }
}
