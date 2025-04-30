use serde::{Deserialize, Serialize};
use std::error::Error;

// Define structures for MediaWiki API responses
#[derive(Debug, Deserialize)]
struct WikiResponse {
    query: Option<QueryResult>,
}

#[derive(Debug, Deserialize)]
struct QueryResult {
    pages: std::collections::HashMap<String, PageData>,
}

#[derive(Debug, Deserialize)]
struct PageData {
    pageid: Option<i64>,
    title: Option<String>,
    extract: Option<String>,
    thumbnail: Option<ThumbnailInfo>,
    fullurl: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ThumbnailInfo {
    source: String,
    width: i32,
    height: i32,
}

// Add Clone trait to WikiArticle
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WikiArticle {
    pub title: String,
    pub description: String,
    pub url: String,
    pub image_url: Option<String>,
}

// Function to search GTA Wiki for a specific term
pub async fn search_gta_wiki(search_term: &str) -> Result<Vec<WikiArticle>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();
    
    println!("Searching GTA Wiki for: {}", search_term);
    
    // Build the API URL for searching
    let api_url = format!(
        "https://gta.fandom.com/api.php?action=query&format=json&list=search&srsearch={}&srlimit=5",
        search_term
    );
    
    // Make the search request
    let search_response = client.get(&api_url)
        .header("User-Agent", "GTA Discord Bot/0.1")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    
    // Extract search results
    let search_results = search_response["query"]["search"].as_array()
        .ok_or("No search results found")?;
    
    // Collect the page IDs from search results
    let mut page_ids = Vec::new();
    for result in search_results {
        if let Some(pageid) = result["pageid"].as_i64() {
            page_ids.push(pageid.to_string());
        }
    }
    
    if page_ids.is_empty() {
        return Ok(Vec::new());
    }
    
    // Build the API URL for getting page details
    let page_ids_str = page_ids.join("|");
    let details_url = format!(
        "https://gta.fandom.com/api.php?action=query&format=json&prop=extracts|pageimages|info&exintro=1&explaintext=1&inprop=url&piprop=thumbnail&pithumbsize=300&pageids={}",
        page_ids_str
    );
    
    // Make the details request
    let details_response = client.get(&details_url)
        .header("User-Agent", "GTA Discord Bot/0.1")
        .send()
        .await?
        .json::<WikiResponse>()
        .await?;
    
    // Process the results
    let mut articles = Vec::new();
    
    if let Some(query) = details_response.query {
        for (_, page) in query.pages {
            let title = page.title.unwrap_or_else(|| String::from("Unknown"));
            let description = page.extract.unwrap_or_else(|| String::from("No description available."));
            let url = page.fullurl.unwrap_or_else(|| String::from("https://gta.fandom.com"));
            let image_url = page.thumbnail.map(|t| t.source);
            
            // Truncate description if it's too long
            let short_description = if description.len() > 300 {
                format!("{}...", &description[0..300])
            } else {
                description
            };
            
            articles.push(WikiArticle {
                title,
                description: short_description,
                url,
                image_url,
            });
        }
    }
    
    Ok(articles)
}

// Function to get information about a specific vehicle
pub async fn get_vehicle_info(vehicle_name: &str) -> Result<Option<WikiArticle>, Box<dyn Error + Send + Sync>> {
    let articles = search_gta_wiki(&format!("GTA Online {}", vehicle_name)).await?;
    
    if articles.is_empty() {
        Ok(None)
    } else {
        Ok(Some(articles[0].clone()))
    }
}

// Function to get information about a property
pub async fn get_property_info(property_name: &str) -> Result<Option<WikiArticle>, Box<dyn Error + Send + Sync>> {
    let articles = search_gta_wiki(&format!("GTA Online property {}", property_name)).await?;
    
    if articles.is_empty() {
        Ok(None)
    } else {
        Ok(Some(articles[0].clone()))
    }
}

// Function to get character information
pub async fn get_character_info(character_name: &str) -> Result<Option<WikiArticle>, Box<dyn Error + Send + Sync>> {
    let articles = search_gta_wiki(character_name).await?;
    
    if articles.is_empty() {
        Ok(None)
    } else {
        Ok(Some(articles[0].clone()))
    }
}