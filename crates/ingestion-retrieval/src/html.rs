use std::collections::{HashMap, HashSet};

use ammonia::{Builder, Url, UrlRelative};
use anyhow::{Context, Result, bail};
use dom_smoothie::Readability;
use encoding_rs::{Encoding, UTF_8, WINDOWS_1252};

pub(crate) fn decode_html(bytes: &[u8], content_type: Option<&str>) -> Result<String> {
    let encoding = if let Some((encoding, _)) = Encoding::for_bom(bytes) {
        encoding
    } else if let Some(label) = content_type.and_then(header_charset).map(str::as_bytes).or_else(|| meta_charset(bytes)) {
        Encoding::for_label(label).with_context(|| format!("web page declares unsupported character encoding {}", String::from_utf8_lossy(label)))?
    } else if std::str::from_utf8(bytes).is_ok() {
        UTF_8
    } else {
        WINDOWS_1252
    };
    let (text, _, had_errors) = encoding.decode(bytes);
    if had_errors {
        bail!("web page contains invalid {} text", encoding.name().to_ascii_lowercase());
    }
    Ok(text.into_owned())
}

fn header_charset(content_type: &str) -> Option<&str> {
    content_type.split(';').skip(1).find_map(|parameter| {
        let (name, value) = parameter.trim().split_once('=')?;
        name.trim().eq_ignore_ascii_case("charset").then(|| value.trim().trim_matches(['\"', '\'']))
    })
}

fn meta_charset(bytes: &[u8]) -> Option<&[u8]> {
    let prefix = &bytes[..bytes.len().min(1024)];
    let lowercase: Vec<_> = prefix.iter().map(u8::to_ascii_lowercase).collect();
    let mut offset = 0;
    while let Some(start) = lowercase[offset..].windows(5).position(|window| window == b"<meta").map(|position| offset + position) {
        let end = lowercase[start..].iter().position(|byte| *byte == b'>').map(|position| start + position + 1)?;
        let tag = &lowercase[start..end];
        if let Some(position) = tag.windows(7).position(|window| window == b"charset") {
            let mut value_start = position + 7;
            while tag.get(value_start).is_some_and(u8::is_ascii_whitespace) {
                value_start += 1;
            }
            if tag.get(value_start) == Some(&b'=') {
                value_start += 1;
                while tag.get(value_start).is_some_and(u8::is_ascii_whitespace) {
                    value_start += 1;
                }
                let quote = tag.get(value_start).copied().filter(|byte| matches!(byte, b'\"' | b'\''));
                value_start += usize::from(quote.is_some());
                let value_end = tag[value_start..]
                    .iter()
                    .position(|byte| quote.map_or_else(|| byte.is_ascii_whitespace() || matches!(byte, b';' | b'>' | b'\"' | b'\''), |quote| *byte == quote))
                    .map(|length| value_start + length)
                    .unwrap_or(tag.len());
                return Some(&prefix[start + value_start..start + value_end]);
            }
        }
        offset = end;
    }
    None
}

pub(crate) fn clean(source: &str, url: &str) -> Result<(String, String)> {
    let (title, content) = extract(source, url);
    let base_url = Url::parse(url).context("web page URL is invalid")?;
    let cleaned = sanitizer(base_url).clean(&content).to_string();
    let escaped_title = title.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    Ok((
        title,
        format!("<!doctype html>\n<html>\n<head><meta charset=\"utf-8\"><title>{escaped_title}</title></head>\n<body>\n{cleaned}\n</body>\n</html>\n"),
    ))
}

fn extract(source: &str, url: &str) -> (String, String) {
    let Ok(mut readability) = Readability::new(source, Some(url), None) else {
        return (String::new(), source.to_owned());
    };
    let fallback_title = readability.get_article_title().to_string();
    match readability.parse() {
        Ok(article) => (article.title, article.content.to_string()),
        Err(_) => (fallback_title, source.to_owned()),
    }
}

fn sanitizer(base_url: Url) -> Builder<'static> {
    let tags = HashSet::from([
        "a",
        "article",
        "b",
        "blockquote",
        "br",
        "caption",
        "code",
        "dd",
        "del",
        "dl",
        "dt",
        "em",
        "figcaption",
        "figure",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "hr",
        "i",
        "img",
        "li",
        "main",
        "ol",
        "p",
        "pre",
        "section",
        "strong",
        "sub",
        "sup",
        "table",
        "tbody",
        "td",
        "tfoot",
        "th",
        "thead",
        "time",
        "tr",
        "ul",
    ]);
    let attributes = HashMap::from([
        ("a", HashSet::from(["href"])),
        ("img", HashSet::from(["alt", "src"])),
        ("ol", HashSet::from(["start"])),
        ("td", HashSet::from(["colspan", "rowspan"])),
        ("th", HashSet::from(["colspan", "rowspan", "scope"])),
        ("time", HashSet::from(["datetime"])),
    ]);
    let discarded_content = HashSet::from([
        "audio", "button", "canvas", "footer", "form", "head", "iframe", "nav", "noscript", "script", "style", "svg", "template", "video",
    ]);
    let mut builder = Builder::new();
    builder
        .tags(tags)
        .tag_attributes(attributes)
        .generic_attributes(HashSet::new())
        .clean_content_tags(discarded_content)
        .link_rel(None)
        .url_relative(UrlRelative::RewriteWithBase(base_url));
    builder
}

#[cfg(test)]
mod tests {
    use super::clean;

    #[test]
    fn keeps_content_and_removes_page_chrome() {
        let source = r#"<html><head><title>Blue Line</title><script>alert("no")</script></head><body><nav>Site navigation</nav><article><h1 class="headline">Blue Line</h1><p>The Blue Line connects the northern and southern districts with frequent public transport service throughout the day.</p><p>Passengers can transfer to regional rail services at Central Station and use accessible entrances on both sides.</p><p>See the <a href="/map" onclick="bad()">network map</a> for the complete route and interchange information.</p></article><footer>Footer links</footer></body></html>"#;
        let (title, result) = clean(source, "https://example.com/lines/blue").unwrap();
        assert_eq!(title, "Blue Line");
        assert!(result.contains("<title>Blue Line</title>"));
        assert!(result.contains("The Blue Line connects"));
        assert!(result.contains("href=\"https://example.com/map\""));
        assert!(!result.contains("Site navigation"));
        assert!(!result.contains("onclick"));
        assert!(!result.contains("class="));
        assert!(!result.contains("<script"));
    }
}
