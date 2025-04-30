use scraper::{Html, Selector};
use std::error::Error;

#[derive(Debug, Clone)]
pub struct WeeklyUpdate {
    pub title: String,
    pub url: String,
    pub date: String,
    pub bonuses: Vec<String>,
    pub discounts: Vec<String>,
    pub podium_vehicle: Option<String>,
    pub prize_ride: Option<String>,
}

// Struct to hold raw article data extracted from HTML
#[derive(Debug)]
struct RawArticleData {
    title: String,
    url: String,
    date: String,
}

// Function to extract initial article info - runs in a blocking context
fn extract_article_info(html: &str) -> Result<RawArticleData, Box<dyn Error + Send + Sync>> {
    // Parse the HTML
    let document = Html::parse_document(html);
    
    // Find the first article
    let article_selector = Selector::parse("article.news-article").unwrap();
    let first_article = document.select(&article_selector).next()
        .ok_or_else(|| -> Box<dyn Error + Send + Sync> { "No article found".into() })?;
    
    // Extract title
    let title_selector = Selector::parse("h1").unwrap();
    let title = first_article.select(&title_selector).next()
        .map(|el| el.inner_html().trim().to_string())
        .unwrap_or_else(|| String::from("Unknown Title"));
    
    // Extract URL
    let url_selector = Selector::parse("a.button").unwrap();
    let url = first_article.select(&url_selector).next()
        .map(|el| el.value().attr("href").unwrap_or(""))
        .map(|href| format!("https://www.rockstargames.com{}", href))
        .unwrap_or_else(|| String::from("https://www.rockstargames.com/newswire"));
    
    // Extract date
    let date_selector = Selector::parse("time").unwrap();
    let date = first_article.select(&date_selector).next()
        .map(|el| el.inner_html().trim().to_string())
        .unwrap_or_else(|| String::from("Unknown Date"));
    
    Ok(RawArticleData { title, url, date })
}

// Function to extract content from article page - runs in a blocking context
fn extract_article_content(html: &str) -> Result<(Vec<String>, Vec<String>, Option<String>, Option<String>), Box<dyn Error + Send + Sync>> {
    // Parse the HTML
    let document = Html::parse_document(html);
    
    // Find the content
    let content_selector = Selector::parse("div.page-content").unwrap();
    let content = document.select(&content_selector).next()
        .ok_or_else(|| -> Box<dyn Error + Send + Sync> { "Article content not found".into() })?;
    
    // Extract paragraphs
    let paragraph_selector = Selector::parse("p").unwrap();
    let paragraphs: Vec<String> = content.select(&paragraph_selector)
        .map(|p| p.text().collect::<Vec<_>>().join("").trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    
    // Process paragraphs
    let mut bonuses = Vec::new();
    let mut discounts = Vec::new();
    let mut podium_vehicle = None;
    let mut prize_ride = None;
    
    for paragraph in &paragraphs {
        let lower_p = paragraph.to_lowercase();
        
        // Check for bonuses
        if lower_p.contains("2x") || lower_p.contains("3x") || lower_p.contains("bonus") || 
           lower_p.contains("double") || lower_p.contains("triple") {
            bonuses.push(paragraph.clone());
        }
        
        // Check for discounts
        if lower_p.contains("discount") || lower_p.contains("% off") || lower_p.contains("sale") {
            discounts.push(paragraph.clone());
        }
        
        // Check for podium vehicle
        if lower_p.contains("podium") || lower_p.contains("diamond casino") {
            podium_vehicle = Some(paragraph.clone());
        }
        
        // Check for prize ride
        if lower_p.contains("prize ride") || lower_p.contains("ls car meet") {
            prize_ride = Some(paragraph.clone());
        }
    }
    
    Ok((bonuses, discounts, podium_vehicle, prize_ride))
}

pub async fn get_latest_weekly_update() -> Result<WeeklyUpdate, Box<dyn Error + Send + Sync>> {
    // The URL for Rockstar Newswire filtered by GTA Online tag
    let url = "https://www.rockstargames.com/newswire/tags/702";
    
    println!("Fetching GTA Online weekly updates...");
    
    // Make the request
    let response = reqwest::get(url).await?;
    let html_content = response.text().await?;
    
    // Run HTML parsing in a blocking task to avoid Send issues
    let article_data = tokio::task::spawn_blocking(move || {
        extract_article_info(&html_content)
    }).await??;
    
    // Fetch the article page
    println!("Fetching article details from: {}", article_data.url);
    let article_response = reqwest::get(&article_data.url).await?;
    let article_html = article_response.text().await?;
    
    // Run content parsing in a blocking task
    let (bonuses, discounts, podium_vehicle, prize_ride) = tokio::task::spawn_blocking(move || {
        extract_article_content(&article_html)
    }).await??;
    
    // Format the results
    Ok(WeeklyUpdate {
        title: article_data.title,
        url: article_data.url,
        date: article_data.date,
        bonuses,
        discounts,
        podium_vehicle,
        prize_ride,
    })
}

// Function to check GTA Online server status
pub async fn check_server_status() -> Result<String, Box<dyn Error + Send + Sync>> {
    let status_url = "https://support.rockstargames.com/services/status.json";
    
    // Make the request
    let response = reqwest::get(status_url).await?;
    
    if !response.status().is_success() {
        return Ok(String::from("Unable to fetch Rockstar server status."));
    }
    
    let status_json: serde_json::Value = response.json().await?;
    
    // Extract GTA Online status
    let gta_online_status = status_json
        .get("statuses")
        .and_then(|statuses| statuses.as_array())
        .and_then(|status_array| {
            status_array.iter().find(|&service| {
                service.get("name")
                    .and_then(|name| name.as_str())
                    .map_or(false, |name_str| name_str.contains("GTA Online"))
            })
        });
    
    if let Some(status) = gta_online_status {
        let status_string = status.get("status_tag")
            .and_then(|tag| tag.as_str())
            .unwrap_or("Unknown");
        
        let message = status.get("message")
            .and_then(|msg| msg.as_str())
            .unwrap_or("");
        
        Ok(format!("GTA Online Status: {}\n{}", status_string, message))
    } else {
        Ok(String::from("GTA Online status information not found."))
    }
}