use crate::context_loader;
use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProfileSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub value: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FetchProfileDataParams {
    pub profile_sources: Option<Vec<ProfileSource>>,
    pub content: Option<String>,
    pub working_style: Option<String>,
    pub collaboration_prefs: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FetchResult {
    pub success: bool,
    pub prompt: String,
    pub fetched_content: Vec<FetchedContent>,
}

#[derive(Debug, Serialize)]
pub struct FetchedContent {
    pub source_type: String,
    pub source_value: String,
    pub content: String,
}

pub async fn fetch_profile_data(params: FetchProfileDataParams) -> Result<FetchResult> {
    let mut fetched_content = Vec::new();

    // Handle profile sources - fetch the data
    if let Some(sources) = params.profile_sources {
        for source in sources {
            let content = fetch_profile_source(&source).await?;
            fetched_content.push(FetchedContent {
                source_type: source.source_type.clone(),
                source_value: source.value.clone(),
                content,
            });
        }
    }

    // Handle direct content
    if let Some(content) = params.content {
        fetched_content.push(FetchedContent {
            source_type: "text".to_string(),
            source_value: "direct".to_string(),
            content,
        });
    }

    // Handle working style
    if let Some(style) = params.working_style {
        fetched_content.push(FetchedContent {
            source_type: "working_style".to_string(),
            source_value: "direct".to_string(),
            content: style,
        });
    }

    // Handle collaboration preferences
    if let Some(prefs) = params.collaboration_prefs {
        fetched_content.push(FetchedContent {
            source_type: "collaboration_prefs".to_string(),
            source_value: "direct".to_string(),
            content: prefs,
        });
    }

    // Load config to get human name
    let human_name = context_loader::load_config()
        .ok()
        .map(|config| config.human.name.clone())
        .unwrap_or_else(|| "the user".to_string());

    // Read existing profile
    let profile_content = crate::sparkle_paths::get_sparkle_dir(None)
        .ok()
        .and_then(|sparkle_dir| {
            let profile_path = sparkle_dir.join("collaborator-profile.md");
            std::fs::read_to_string(profile_path).ok()
        })
        .unwrap_or_else(|| "[No existing profile]".to_string());

    let prompt = format!(
        "Evaluate how this fetched data fits into {}'s collaborator profile.\n\n\
        ## Current Profile:\n{}\n\n\
        ## Fetched Data:\n\
        Review the fetched_content below and decide what to add and where. \
        Use the update_collaborator_profile tool to write your changes. \
        Make sure to get their input and consent if they like it!",
        human_name, profile_content
    );

    Ok(FetchResult {
        success: true,
        prompt,
        fetched_content,
    })
}

async fn fetch_profile_source(source: &ProfileSource) -> Result<String> {
    match source.source_type.as_str() {
        "github" => fetch_github_profile(&source.value).await,
        "blog" => fetch_blog_rss(&source.value).await,
        "url" => fetch_url_content(&source.value).await,
        "text" => Ok(source.value.clone()),
        _ => anyhow::bail!("Unknown source type: {}", source.source_type),
    }
}

async fn fetch_github_profile(username: &str) -> Result<String> {
    let client = reqwest::Client::new();

    // Fetch user profile
    let user_url = format!("https://api.github.com/users/{}", username);
    let user_resp = client
        .get(&user_url)
        .header("User-Agent", "Sparkle-MCP")
        .send()
        .await
        .context("Failed to fetch GitHub user")?;

    if !user_resp.status().is_success() {
        anyhow::bail!("GitHub user '{}' not found", username);
    }

    let user: serde_json::Value = user_resp.json().await?;

    // Fetch repositories
    let repos_url = format!(
        "https://api.github.com/users/{}/repos?sort=updated&per_page=100",
        username
    );
    let repos_resp = client
        .get(&repos_url)
        .header("User-Agent", "Sparkle-MCP")
        .send()
        .await
        .context("Failed to fetch GitHub repos")?;

    let repos: Vec<serde_json::Value> = repos_resp.json().await?;

    // Extract profile info
    let name = user["name"].as_str().unwrap_or(username);
    let bio = user["bio"].as_str().unwrap_or("");
    let company = user["company"].as_str().unwrap_or("");
    let location = user["location"].as_str().unwrap_or("");
    let blog = user["blog"].as_str().unwrap_or("");

    // Count languages
    let mut language_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for repo in &repos {
        if let Some(lang) = repo["language"].as_str() {
            *language_counts.entry(lang.to_string()).or_insert(0) += 1;
        }
    }

    // Get top 3 languages
    let mut langs: Vec<_> = language_counts.into_iter().collect();
    langs.sort_by(|a, b| b.1.cmp(&a.1));
    let top_languages: Vec<String> = langs.iter().take(3).map(|(lang, _)| lang.clone()).collect();

    // Format summary
    let mut summary = format!("## GitHub Profile: {}\n\n", name);

    if !bio.is_empty() {
        summary.push_str(&format!("{}\n\n", bio));
    }

    if !company.is_empty() || !location.is_empty() {
        summary.push_str("**Details:**\n");
        if !company.is_empty() {
            summary.push_str(&format!("- Company: {}\n", company));
        }
        if !location.is_empty() {
            summary.push_str(&format!("- Location: {}\n", location));
        }
        if !blog.is_empty() {
            summary.push_str(&format!("- Website: {}\n", blog));
        }
        summary.push('\n');
    }

    summary.push_str(&format!("**GitHub Activity:**\n"));
    summary.push_str(&format!("- {} public repositories\n", repos.len()));

    if !top_languages.is_empty() {
        summary.push_str(&format!(
            "- Primary languages: {}\n",
            top_languages.join(", ")
        ));
    }

    Ok(summary)
}

async fn fetch_blog_rss(url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "Sparkle-MCP")
        .send()
        .await
        .context("Failed to fetch blog RSS")?;

    if !resp.status().is_success() {
        anyhow::bail!("Failed to fetch blog RSS: {}", resp.status());
    }

    let content = resp.bytes().await?;
    let feed = feed_rs::parser::parse(&content[..]).context("Failed to parse RSS/Atom feed")?;

    let mut summary = format!(
        "## Blog: {}\n\n",
        feed.title
            .map(|t| t.content)
            .unwrap_or_else(|| "Blog".to_string())
    );

    if let Some(description) = feed.description {
        summary.push_str(&format!("{}\n\n", description.content));
    }

    summary.push_str("**Recent Posts:**\n\n");

    // Get up to 10 most recent entries
    let entries: Vec<_> = feed.entries.iter().take(10).collect();

    for entry in entries {
        let title = entry
            .title
            .as_ref()
            .map(|t| t.content.as_str())
            .unwrap_or("Untitled");

        let link = entry.links.first().map(|l| l.href.as_str()).unwrap_or("");

        summary.push_str(&format!("- [{}]({})\n", title, link));

        if let Some(published) = &entry.published {
            summary.push_str(&format!(
                "  *Published: {}*\n",
                published.format("%Y-%m-%d")
            ));
        }

        if let Some(summary_text) = &entry.summary {
            let truncated = if summary_text.content.len() > 200 {
                format!("{}...", &summary_text.content[..200])
            } else {
                summary_text.content.clone()
            };
            summary.push_str(&format!("  {}\n", truncated));
        }

        summary.push('\n');
    }

    Ok(summary)
}

async fn fetch_url_content(url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "Sparkle-MCP")
        .send()
        .await
        .context("Failed to fetch URL")?;

    if !resp.status().is_success() {
        anyhow::bail!("Failed to fetch URL: {}", resp.status());
    }

    let text = resp.text().await?;
    Ok(text)
}
