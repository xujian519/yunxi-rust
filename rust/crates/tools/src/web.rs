use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::json;

// --- Input/Output types ---

#[derive(Debug, serde::Deserialize)]
pub(crate) struct WebFetchInput {
    pub url: String,
    pub prompt: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct WebSearchInput {
    pub query: String,
    pub allowed_domains: Option<Vec<String>>,
    pub blocked_domains: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct WebFetchOutput {
    pub bytes: usize,
    pub code: u16,
    #[serde(rename = "codeText")]
    pub code_text: String,
    pub result: String,
    #[serde(rename = "durationMs")]
    pub duration_ms: u128,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct WebSearchOutput {
    pub query: String,
    pub results: Vec<WebSearchResultItem>,
    #[serde(rename = "durationSeconds")]
    pub duration_seconds: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum WebSearchResultItem {
    SearchResult {
        tool_use_id: String,
        content: Vec<SearchHit>,
    },
    Commentary(String),
}

#[derive(Debug, Serialize)]
pub(crate) struct SearchHit {
    pub title: String,
    pub url: String,
}

// --- Layer orchestration ---

pub(crate) fn execute_web_search(input: &WebSearchInput) -> Result<WebSearchOutput, String> {
    execute_web_search_impl(input)
}

/// 异步版本：在 `spawn_blocking` 中执行网页搜索，避免阻塞 tokio 运行时。
pub(crate) async fn execute_web_search_async(
    input: WebSearchInput,
) -> Result<WebSearchOutput, String> {
    tokio::task::spawn_blocking(move || execute_web_search_impl(&input))
        .await
        .map_err(|e| format!("spawn_blocking failed: {e}"))?
}

fn execute_web_search_impl(input: &WebSearchInput) -> Result<WebSearchOutput, String> {
    let started = Instant::now();

    let (hits, backend) = search_with_fallback(input)?;

    let hits = apply_filters(hits, input);

    let summary = if hits.is_empty() {
        format!(
            "No web search results matched the query {:?} (backend: {backend}).",
            input.query
        )
    } else {
        let rendered_hits = hits
            .iter()
            .map(|hit| format!("- [{}]({})", hit.title, hit.url))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "Search results for {:?} (backend: {backend}). Include a Sources section.\n{rendered_hits}",
            input.query
        )
    };

    Ok(WebSearchOutput {
        query: input.query.clone(),
        results: vec![
            WebSearchResultItem::Commentary(summary),
            WebSearchResultItem::SearchResult {
                tool_use_id: String::from("web_search_1"),
                content: hits,
            },
        ],
        duration_seconds: started.elapsed().as_secs_f64(),
        backend: Some(backend),
    })
}

fn search_with_fallback(input: &WebSearchInput) -> Result<(Vec<SearchHit>, String), String> {
    // Layer 1: User-configured custom backend (SearXNG etc.) — takes priority if explicitly set
    if let Ok(base) = std::env::var("YUNXI_WEB_SEARCH_BASE_URL") {
        if !base.is_empty() {
            match search_custom_backend(input) {
                Ok(hits) if !hits.is_empty() => return Ok((hits, "custom-backend".to_string())),
                Err(e) => eprintln!("[web] custom backend failed: {e}, falling back"),
                _ => {}
            }
        }
    }

    // Layer 2: AnySearch — zero API key, ip/legal/cn vertical domains for patent
    if !skip_anysearch() {
        match search_anysearch(input) {
            Ok(hits) if !hits.is_empty() => return Ok((hits, "anysearch".to_string())),
            Err(e) => eprintln!("[web] anysearch failed: {e}, falling back"),
            _ => {}
        }
    }

    // Layer 3: Tavily API (AI-optimized, requires API key)
    if let Ok(key) = std::env::var("TAVILY_API_KEY") {
        if !key.is_empty() {
            match search_tavily(&input.query, &key) {
                Ok(hits) if !hits.is_empty() => return Ok((hits, "tavily".to_string())),
                Err(e) => eprintln!("[web] tavily failed: {e}, falling back"),
                _ => eprintln!("[web] tavily returned empty results, falling back"),
            }
        }
    }

    // Layer 4: headless_chrome browser search (sends real Chrome to DDG)
    #[cfg(feature = "browser-fallback")]
    {
        match search_via_browser(&input.query) {
            Ok(hits) if !hits.is_empty() => return Ok((hits, "browser-chrome".to_string())),
            Err(e) => eprintln!("[web] browser fallback failed: {e}, falling back"),
            _ => {}
        }
    }

    // Layer 5: DuckDuckGo HTML scraping (built-in, zero dependency)
    match search_duckduckgo_html(&input.query) {
        Ok(hits) if !hits.is_empty() => return Ok((hits, "duckduckgo-html".to_string())),
        Ok(_) => return Ok((vec![], "duckduckgo-html".to_string())),
        Err(e) => Err(format!("all search backends failed: {e}")),
    }
}

fn apply_filters(mut hits: Vec<SearchHit>, input: &WebSearchInput) -> Vec<SearchHit> {
    if let Some(allowed) = input.allowed_domains.as_ref() {
        hits.retain(|hit| host_matches_list(&hit.url, allowed));
    }
    if let Some(blocked) = input.blocked_domains.as_ref() {
        hits.retain(|hit| !host_matches_list(&hit.url, blocked));
    }
    dedupe_hits(&mut hits);
    hits.truncate(8);
    hits
}

// --- Layer 1: Tavily API ---

fn search_tavily(query: &str, api_key: &str) -> Result<Vec<SearchHit>, String> {
    let client = build_http_client()?;
    let response = client
        .post("https://api.tavily.com/search")
        .json(&json!({
            "api_key": api_key,
            "query": query,
            "max_results": 8,
            "search_depth": "basic",
        }))
        .send()
        .map_err(|e| format!("tavily request failed: {e}"))?;

    let status = response.status();
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err("tavily rate limited (429)".to_string());
    }
    if status == reqwest::StatusCode::PAYMENT_REQUIRED {
        return Err("tavily quota exhausted (402)".to_string());
    }
    if !status.is_success() {
        return Err(format!("tavily HTTP {}", status.as_u16()));
    }

    let body: serde_json::Value = response
        .json()
        .map_err(|e| format!("tavily json parse: {e}"))?;

    let results = body["results"]
        .as_array()
        .ok_or("tavily: missing results array")?;

    let hits: Vec<SearchHit> = results
        .iter()
        .filter_map(|r| {
            Some(SearchHit {
                title: r["title"].as_str()?.to_string(),
                url: r["url"].as_str()?.to_string(),
            })
        })
        .collect();

    Ok(hits)
}

// --- Layer 3: AnySearch — zero API key, vertical domain support ---

const ANYSEARCH_ENDPOINT: &str = "https://api.anysearch.com/mcp";

fn skip_anysearch() -> bool {
    std::env::var("YUNXI_SKIP_ANYSEARCH").is_ok()
}

fn anysearch_api_key() -> Option<String> {
    std::env::var("ANYSEARCH_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
}

fn is_patent_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    // CN patent number patterns
    let has_cn_patent = lower.contains("cn")
        && (lower.chars().filter(|c| c.is_ascii_digit()).count() >= 8
            || lower.contains("a")
            || lower.contains("b")
            || lower.contains("u"));
    let patent_keywords = [
        "patent",
        "专利",
        "claim",
        "权利要求",
        "invention",
        "发明",
        "infringement",
        "侵权",
        "novelty",
        "新颖性",
        "prior art",
        "现有技术",
        "examiner",
        "审查",
        "ipc",
        "cpc",
        "trademark",
        "商标",
    ];
    has_cn_patent || patent_keywords.iter().any(|kw| lower.contains(kw))
}

fn search_anysearch(input: &WebSearchInput) -> Result<Vec<SearchHit>, String> {
    let client = build_http_client()?;
    let query = &input.query;

    // Auto-detect patent/IP domain
    let (domain, zone) = if is_patent_query(query) {
        (Some("ip"), Some("cn"))
    } else {
        (None, None)
    };

    let mut args = serde_json::Map::new();
    args.insert("query".to_string(), json!(query));
    args.insert("max_results".to_string(), json!(8));
    if let Some(d) = domain {
        args.insert("domain".to_string(), json!(d));
    }
    if let Some(z) = zone {
        args.insert("zone".to_string(), json!(z));
    }
    if args.contains_key("domain") {
        args.insert("freshness".to_string(), "year".into());
    }

    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "search",
            "arguments": args,
        }
    });

    let mut req = client.post(ANYSEARCH_ENDPOINT).json(&payload);
    if let Some(key) = anysearch_api_key() {
        req = req.header("Authorization", format!("Bearer {key}"));
    }

    let response = req.send().map_err(|e| format!("anysearch request: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("anysearch HTTP {}", response.status().as_u16()));
    }

    let body: serde_json::Value = response
        .json()
        .map_err(|e| format!("anysearch json parse: {e}"))?;

    if body.get("error").is_some() {
        let msg = body["error"]["message"].as_str().unwrap_or("unknown error");
        return Err(format!("anysearch api error: {msg}"));
    }

    let content = body["result"]["content"]
        .as_array()
        .ok_or("anysearch: missing content array")?;

    let text = content
        .iter()
        .find(|item| item["type"].as_str() == Some("text"))
        .and_then(|item| item["text"].as_str())
        .unwrap_or("");

    // Parse markdown/text to extract links (format: "- [title](url)" or "- title -> url")
    parse_anysearch_results(text)
}

fn parse_anysearch_results(text: &str) -> Result<Vec<SearchHit>, String> {
    let mut hits = Vec::new();

    // Parse AnySearch format: "### N. title\n- **URL**: url\n..."
    let mut title: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();

        // Title line: "### N. Title"
        if trimmed.starts_with("### ") {
            // Strip "### N. " prefix
            let t = trimmed
                .trim_start_matches('#')
                .trim()
                .chars()
                .skip_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .trim()
                .trim_start_matches('.')
                .trim()
                .to_string();
            if !t.is_empty() {
                title = Some(t);
            }
            continue;
        }

        // URL line: "- **URL**: https://..."
        if trimmed.starts_with("- **URL**:") || trimmed.starts_with("- **URL** :") {
            if let Some(t) = title.take() {
                let url = trimmed
                    .trim_start_matches('-')
                    .trim()
                    .trim_start_matches("**URL**")
                    .trim()
                    .trim_start_matches(':')
                    .trim()
                    .to_string();
                if url.starts_with("http://") || url.starts_with("https://") {
                    hits.push(SearchHit { title: t, url });
                }
            }
            continue;
        }
    }

    // Fallback: standard markdown links "- [title](url)"
    if hits.is_empty() {
        let mut remaining = text;
        while let Some(start) = remaining.find('[') {
            let end_bracket = remaining[start..].find("](");
            let Some(end_bracket_idx) = end_bracket else {
                remaining = &remaining[start + 1..];
                continue;
            };
            let t = &remaining[start + 1..start + end_bracket_idx];
            let url_start = start + end_bracket_idx + 2;
            let url_end = remaining[url_start..].find(')');
            let Some(url_end_idx) = url_end else {
                remaining = &remaining[start + 1..];
                continue;
            };
            let u = &remaining[url_start..url_start + url_end_idx];
            let t = t.trim().to_string();
            let u = u.trim().to_string();
            if !t.is_empty() && (u.starts_with("http://") || u.starts_with("https://")) {
                hits.push(SearchHit { title: t, url: u });
            }
            remaining = &remaining[url_start + url_end_idx + 1..];
        }
    }

    Ok(hits)
}

// --- Layer 2: Custom backend (SearXNG / user-defined) ---

fn search_custom_backend(input: &WebSearchInput) -> Result<Vec<SearchHit>, String> {
    let client = build_http_client()?;
    let search_url = build_search_url(&input.query)?;
    let response = client.get(search_url).send().map_err(|e| e.to_string())?;

    let html = response.text().map_err(|e| e.to_string())?;
    let mut hits = extract_search_hits(&html);
    if hits.is_empty() {
        hits = extract_search_hits_from_generic_links(&html);
    }
    Ok(hits)
}

// --- Layer 3: headless_chrome browser search ---

#[cfg(feature = "browser-fallback")]
fn search_via_browser(query: &str) -> Result<Vec<SearchHit>, String> {
    use headless_chrome::{Browser, LaunchOptions};

    let launcher = LaunchOptions::default_builder()
        .headless(true)
        .sandbox(false)
        .window_size(Some((1280, 720)))
        .build()
        .map_err(|e| format!("browser launch config: {e}"))?;

    let browser = Browser::new(launcher).map_err(|e| format!("browser start: {e}"))?;
    let tab = browser.new_tab().map_err(|e| format!("browser tab: {e}"))?;

    let search_url = format!("https://html.duckduckgo.com/html/?q={}", url_encode(query));
    tab.navigate_to(&search_url)
        .map_err(|e| format!("browser navigate: {e}"))?;

    tab.wait_until_navigated()
        .map_err(|e| format!("browser wait: {e}"))?;

    let html = tab
        .get_content()
        .map_err(|e| format!("browser content: {e}"))?;

    let _ = browser; // drop closes browser

    let mut hits = extract_search_hits(&html);
    if hits.is_empty() {
        hits = extract_search_hits_from_generic_links(&html);
    }
    Ok(hits)
}

#[cfg(feature = "browser-fallback")]
fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

// --- Layer 4: DuckDuckGo HTML scraping (always available) ---

fn search_duckduckgo_html(query: &str) -> Result<Vec<SearchHit>, String> {
    let client = build_http_client()?;
    let search_url = ddg_search_url(query)?;
    let response = client.get(search_url).send().map_err(|e| e.to_string())?;

    let html = response.text().map_err(|e| e.to_string())?;
    let mut hits = extract_search_hits(&html);
    if hits.is_empty() {
        hits = extract_search_hits_from_generic_links(&html);
    }
    Ok(hits)
}

fn ddg_search_url(query: &str) -> Result<reqwest::Url, String> {
    let mut url = reqwest::Url::parse("https://html.duckduckgo.com/html/")
        .map_err(|error| error.to_string())?;
    url.query_pairs_mut().append_pair("q", query);
    Ok(url)
}

// --- WebFetch (unchanged) ---

pub(crate) fn execute_web_fetch(input: &WebFetchInput) -> Result<WebFetchOutput, String> {
    execute_web_fetch_impl(input)
}

/// 异步版本：在 `spawn_blocking` 中执行网页抓取，避免阻塞 tokio 运行时。
pub(crate) async fn execute_web_fetch_async(
    input: WebFetchInput,
) -> Result<WebFetchOutput, String> {
    tokio::task::spawn_blocking(move || execute_web_fetch_impl(&input))
        .await
        .map_err(|e| format!("spawn_blocking failed: {e}"))?
}

fn execute_web_fetch_impl(input: &WebFetchInput) -> Result<WebFetchOutput, String> {
    let started = Instant::now();
    let client = build_http_client()?;
    let request_url = normalize_fetch_url(&input.url)?;
    let response = client
        .get(request_url.clone())
        .send()
        .map_err(|error| error.to_string())?;

    let status = response.status();
    let final_url = response.url().to_string();
    let code = status.as_u16();
    let code_text = status.canonical_reason().unwrap_or("Unknown").to_string();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let body = response.text().map_err(|error| error.to_string())?;
    let bytes = body.len();
    let normalized = normalize_fetched_content(&body, &content_type);
    let result = summarize_web_fetch(&final_url, &input.prompt, &normalized, &body, &content_type);

    Ok(WebFetchOutput {
        bytes,
        code,
        code_text,
        result,
        duration_ms: started.elapsed().as_millis(),
        url: final_url,
    })
}

// --- Internal helpers ---

fn build_http_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent("yunxi-rust-tools/0.1")
        .build()
        .map_err(|error| error.to_string())
}

fn normalize_fetch_url(url: &str) -> Result<String, String> {
    let parsed = reqwest::Url::parse(url).map_err(|error| error.to_string())?;
    if parsed.scheme() == "http" {
        let host = parsed.host_str().unwrap_or_default();
        if host != "localhost" && host != "127.0.0.1" && host != "::1" {
            let mut upgraded = parsed;
            upgraded
                .set_scheme("https")
                .map_err(|()| String::from("failed to upgrade URL to https"))?;
            return Ok(upgraded.to_string());
        }
    }
    Ok(parsed.to_string())
}

fn build_search_url(query: &str) -> Result<reqwest::Url, String> {
    if let Ok(base) = std::env::var("YUNXI_WEB_SEARCH_BASE_URL") {
        let mut url = reqwest::Url::parse(&base).map_err(|error| error.to_string())?;
        url.query_pairs_mut().append_pair("q", query);
        return Ok(url);
    }

    let mut url = reqwest::Url::parse("https://html.duckduckgo.com/html/")
        .map_err(|error| error.to_string())?;
    url.query_pairs_mut().append_pair("q", query);
    Ok(url)
}

fn normalize_fetched_content(body: &str, content_type: &str) -> String {
    if content_type.contains("html") {
        html_to_text(body)
    } else {
        body.trim().to_string()
    }
}

fn summarize_web_fetch(
    url: &str,
    prompt: &str,
    content: &str,
    raw_body: &str,
    content_type: &str,
) -> String {
    let lower_prompt = prompt.to_lowercase();
    let compact = collapse_whitespace(content);

    let detail = if lower_prompt.contains("title") {
        extract_title(content, raw_body, content_type).map_or_else(
            || preview_text(&compact, 600),
            |title| format!("Title: {title}"),
        )
    } else if lower_prompt.contains("summary") || lower_prompt.contains("summarize") {
        preview_text(&compact, 900)
    } else {
        let preview = preview_text(&compact, 900);
        format!("Prompt: {prompt}\nContent preview:\n{preview}")
    };

    format!("Fetched {url}\n{detail}")
}

fn extract_title(content: &str, raw_body: &str, content_type: &str) -> Option<String> {
    if content_type.contains("html") {
        let lowered = raw_body.to_lowercase();
        if let Some(start) = lowered.find("<title>") {
            let after = start + "<title>".len();
            if let Some(end_rel) = lowered[after..].find("</title>") {
                let title =
                    collapse_whitespace(&decode_html_entities(&raw_body[after..after + end_rel]));
                if !title.is_empty() {
                    return Some(title);
                }
            }
        }
    }

    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

pub(crate) fn html_to_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut previous_was_space = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if in_tag => {}
            '&' => {
                text.push('&');
                previous_was_space = false;
            }
            ch if ch.is_whitespace() => {
                if !previous_was_space {
                    text.push(' ');
                    previous_was_space = true;
                }
            }
            _ => {
                text.push(ch);
                previous_was_space = false;
            }
        }
    }

    collapse_whitespace(&decode_html_entities(&text))
}

pub(crate) fn decode_html_entities(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn collapse_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn preview_text(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    let shortened = input.chars().take(max_chars).collect::<String>();
    format!("{}…", shortened.trim_end())
}

fn extract_search_hits(html: &str) -> Vec<SearchHit> {
    let mut hits = Vec::new();
    let mut remaining = html;

    while let Some(anchor_start) = remaining.find("result__a") {
        let after_class = &remaining[anchor_start..];
        let Some(href_idx) = after_class.find("href=") else {
            remaining = &after_class[1..];
            continue;
        };
        let href_slice = &after_class[href_idx + 5..];
        let Some((url, rest)) = extract_quoted_value(href_slice) else {
            remaining = &after_class[1..];
            continue;
        };
        let Some(close_tag_idx) = rest.find('>') else {
            remaining = &after_class[1..];
            continue;
        };
        let after_tag = &rest[close_tag_idx + 1..];
        let Some(end_anchor_idx) = after_tag.find("</a>") else {
            remaining = &after_tag[1..];
            continue;
        };
        let title = html_to_text(&after_tag[..end_anchor_idx]);
        if let Some(decoded_url) = decode_duckduckgo_redirect(&url) {
            hits.push(SearchHit {
                title: title.trim().to_string(),
                url: decoded_url,
            });
        }
        remaining = &after_tag[end_anchor_idx + 4..];
    }

    hits
}

fn extract_search_hits_from_generic_links(html: &str) -> Vec<SearchHit> {
    let mut hits = Vec::new();
    let mut remaining = html;

    while let Some(anchor_start) = remaining.find("<a") {
        let after_anchor = &remaining[anchor_start..];
        let Some(href_idx) = after_anchor.find("href=") else {
            remaining = &after_anchor[2..];
            continue;
        };
        let href_slice = &after_anchor[href_idx + 5..];
        let Some((url, rest)) = extract_quoted_value(href_slice) else {
            remaining = &after_anchor[2..];
            continue;
        };
        let Some(close_tag_idx) = rest.find('>') else {
            remaining = &after_anchor[2..];
            continue;
        };
        let after_tag = &rest[close_tag_idx + 1..];
        let Some(end_anchor_idx) = after_tag.find("</a>") else {
            remaining = &after_anchor[2..];
            continue;
        };
        let title = html_to_text(&after_tag[..end_anchor_idx]);
        if title.trim().is_empty() {
            remaining = &after_tag[end_anchor_idx + 4..];
            continue;
        }
        let decoded_url = decode_duckduckgo_redirect(&url).unwrap_or(url);
        if decoded_url.starts_with("http://") || decoded_url.starts_with("https://") {
            hits.push(SearchHit {
                title: title.trim().to_string(),
                url: decoded_url,
            });
        }
        remaining = &after_tag[end_anchor_idx + 4..];
    }

    hits
}

fn extract_quoted_value(input: &str) -> Option<(String, &str)> {
    let quote = input.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &input[quote.len_utf8()..];
    let end = rest.find(quote)?;
    Some((rest[..end].to_string(), &rest[end + quote.len_utf8()..]))
}

fn decode_duckduckgo_redirect(url: &str) -> Option<String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        return Some(html_entity_decode_url(url));
    }

    let joined = if url.starts_with("//") {
        format!("https:{url}")
    } else if url.starts_with('/') {
        format!("https://duckduckgo.com{url}")
    } else {
        return None;
    };

    let parsed = reqwest::Url::parse(&joined).ok()?;
    if parsed.path() == "/l/" || parsed.path() == "/l" {
        for (key, value) in parsed.query_pairs() {
            if key == "uddg" {
                return Some(html_entity_decode_url(value.as_ref()));
            }
        }
    }
    Some(joined)
}

fn html_entity_decode_url(url: &str) -> String {
    decode_html_entities(url)
}

fn host_matches_list(url: &str, domains: &[String]) -> bool {
    let Ok(parsed) = reqwest::Url::parse(url) else {
        return false;
    };
    let Some(host) = parsed.host_str() else {
        return false;
    };
    let host = host.to_ascii_lowercase();
    domains.iter().any(|domain| {
        let normalized = normalize_domain_filter(domain);
        !normalized.is_empty() && (host == normalized || host.ends_with(&format!(".{normalized}")))
    })
}

fn normalize_domain_filter(domain: &str) -> String {
    let trimmed = domain.trim();
    let candidate = reqwest::Url::parse(trimmed)
        .ok()
        .and_then(|url| url.host_str().map(str::to_string))
        .unwrap_or_else(|| trimmed.to_string());
    candidate
        .trim()
        .trim_start_matches('.')
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

fn dedupe_hits(hits: &mut Vec<SearchHit>) {
    let mut seen = BTreeSet::new();
    hits.retain(|hit| seen.insert(hit.url.clone()));
}
