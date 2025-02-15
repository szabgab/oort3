use oort_simulator::simulation::Code;

pub struct AI {
    pub name: String,
    pub source_code: String,
    pub compiled_code: Code,
}

pub async fn fetch_and_compile(
    http: &reqwest::Client,
    shortcode: &str,
    dev: bool,
) -> anyhow::Result<AI> {
    let (compiler_url, shortcode_url) = if dev {
        ("http://localhost:8081", "http://localhost:8084")
    } else {
        ("https://compiler.oort.rs", "https://shortcode.oort.rs")
    };

    let source_code = if std::fs::metadata(shortcode).is_ok() {
        std::fs::read_to_string(shortcode).unwrap()
    } else {
        log::info!("Fetching {:?}", shortcode);
        http.get(&format!("{shortcode_url}/shortcode/{shortcode}"))
            .send()
            .await?
            .text()
            .await?
    };
    log::info!("Compiling {:?}", shortcode);

    let compiled_code = http
        .post(&format!("{compiler_url}/compile"))
        .body(source_code.clone())
        .send()
        .await?
        .bytes()
        .await?;
    let compiled_code = oort_simulator::vm::precompile(&compiled_code).unwrap();

    Ok(AI {
        name: shortcode.to_string(),
        source_code,
        compiled_code,
    })
}

pub async fn fetch_and_compile_multiple(
    http: &reqwest::Client,
    shortcodes: &[String],
    dev: bool,
) -> anyhow::Result<Vec<AI>> {
    let futures = shortcodes
        .iter()
        .map(|shortcode| fetch_and_compile(http, shortcode, dev));
    let results = futures::future::join_all(futures).await;
    results.into_iter().collect()
}
