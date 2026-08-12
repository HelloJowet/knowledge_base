use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use reqwest::{blocking::Client, header::CONTENT_TYPE};

use crate::html::{clean, decode_html};

#[derive(Debug)]
pub struct FetchedPage {
    pub html: String,
    pub title: String,
    pub url: String,
    pub retrieved_at: String,
}

pub fn fetch_and_clean(url: &str) -> Result<FetchedPage> {
    let client = Client::builder()
        .user_agent("knowledge-base/0.3 (contact@jonasfrei.de)")
        .timeout(Duration::from_secs(30))
        .build()
        .context("could not create HTTP client")?;
    let response = client
        .get(url)
        .send()
        .context("could not fetch URL")?
        .error_for_status()
        .context("web page returned an error")?;
    let final_url = response.url().to_string();
    let content_type = response.headers().get(CONTENT_TYPE).and_then(|value| value.to_str().ok()).map(str::to_owned);
    let bytes = response.bytes().context("could not read web page")?;
    let retrieved_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let source = decode_html(&bytes, content_type.as_deref())?;
    let (title, html) = clean(&source, &final_url)?;

    Ok(FetchedPage {
        html,
        title,
        url: final_url,
        retrieved_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    use crate::html::decode_html;

    #[test]
    fn decodes_declared_legacy_html_and_undeclared_utf8() {
        let legacy = b"<meta http-equiv=\"Content-Type\" content=\"text/html; charset=windows-1252\">S\xf6&#287;\xfctl\xfc\xe7e&#351;me \xa31 \x801";
        let decoded = decode_html(legacy, Some("text/html")).expect("HTML should decode");
        assert!(decoded.contains("Sö&#287;ütlüçe&#351;me £1 €1"));

        let utf8 = "Söğütlüçeşme ₺1";
        assert_eq!(decode_html(utf8.as_bytes(), None).unwrap(), utf8);
    }

    #[test]
    fn follows_redirect_and_reports_final_url() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let body = "<html><head><title>Final page</title></head><body><article><h1>Final page</h1><p>This sufficiently long paragraph describes public transport information on the final page.</p></article></body></html>";
            let responses = [
                format!("HTTP/1.1 302 Found\r\nLocation: http://{address}/final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"),
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                ),
            ];
            for response in responses {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0_u8; 1024];
                let _ = stream.read(&mut request).unwrap();
                stream.write_all(response.as_bytes()).unwrap();
            }
        });

        let fetched = fetch_and_clean(&format!("http://{address}/start")).unwrap();
        server.join().unwrap();

        assert_eq!(fetched.url, format!("http://{address}/final"));
        assert_eq!(fetched.title, "Final page");
        assert!(fetched.retrieved_at.ends_with('Z'));
        assert!(fetched.html.contains("<title>Final page</title>"));
    }
}
