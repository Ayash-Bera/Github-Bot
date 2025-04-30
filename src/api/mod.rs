pub mod gta_wiki;
pub mod weekly_updates;

// Re-export common types and functions for easier imports
pub use gta_wiki::{WikiArticle, search_gta_wiki, get_vehicle_info};
// Removed unused exports: get_property_info, get_character_info
pub use weekly_updates::{get_latest_weekly_update, check_server_status};
// Removed unused export: WeeklyUpdate