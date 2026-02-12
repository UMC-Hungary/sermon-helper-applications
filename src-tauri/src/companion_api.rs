//! Companion HTTP API integration
//!
//! This module provides functionality to communicate with Bitfocus Companion's
//! HTTP API to programmatically create buttons and pages.

use serde::Serialize;

/// Default Companion API port
pub const DEFAULT_COMPANION_PORT: u16 = 8000;

/// Companion API client
pub struct CompanionApi {
    base_url: String,
}

/// Button style configuration
#[derive(Debug, Clone, Serialize)]
pub struct ButtonStyle {
    pub text: String,
    pub size: String,
    pub color: u32,
    pub bgcolor: u32,
}

impl CompanionApi {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            base_url: format!("http://{}:{}", host, port),
        }
    }

    /// Check if Companion is running and accessible
    pub async fn check_connection(&self) -> Result<bool, String> {
        let client = reqwest::Client::new();

        // Try multiple endpoints that might work across different Companion versions
        let endpoints = [
            "/api/version",      // Companion 3.x
            "/api",              // General API check
            "/",                 // Web UI check
        ];

        for endpoint in endpoints {
            match client
                .get(format!("{}{}", self.base_url, endpoint))
                .timeout(std::time::Duration::from_secs(3))
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        log::info!("Companion connected via {}", endpoint);
                        return Ok(true);
                    }
                }
                Err(e) => {
                    log::debug!("Companion check failed for {}: {}", endpoint, e);
                }
            }
        }

        Ok(false)
    }

    /// Set button style at a specific location
    pub async fn set_button_style(
        &self,
        page: u32,
        row: u32,
        column: u32,
        style: &ButtonStyle,
    ) -> Result<(), String> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/api/location/{}/{}/{}/style",
            self.base_url, page, row, column
        );

        let body = serde_json::json!({
            "text": style.text,
            "size": style.size,
            "color": format!("#{:06x}", style.color),
            "bgcolor": format!("#{:06x}", style.bgcolor),
        });

        client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to set button style: {}", e))?;

        Ok(())
    }
}

/// PPT Selector page layout configuration
pub struct PptSelectorLayout {
    /// The page number to create buttons on
    pub page: u32,
}

impl Default for PptSelectorLayout {
    fn default() -> Self {
        Self {
            page: 1,
        }
    }
}

/// Create PPT selector buttons on a Companion page
pub async fn create_ppt_selector_page(
    api: &CompanionApi,
    layout: &PptSelectorLayout,
) -> Result<(), String> {
    // Row 0: Digits 1-5
    let digit_buttons = [
        (0, 0, "1"),
        (0, 1, "2"),
        (0, 2, "3"),
        (0, 3, "4"),
        (0, 4, "5"),
    ];

    for (row, col, digit) in digit_buttons {
        api.set_button_style(
            layout.page,
            row,
            col,
            &ButtonStyle {
                text: digit.to_string(),
                size: "44".to_string(),
                color: 0xFFFFFF,
                bgcolor: 0x3B82F6, // Blue
            },
        )
        .await?;
    }

    // Row 1: Digits 6-9, 0
    let digit_buttons_2 = [
        (1, 0, "6"),
        (1, 1, "7"),
        (1, 2, "8"),
        (1, 3, "9"),
        (1, 4, "0"),
    ];

    for (row, col, digit) in digit_buttons_2 {
        api.set_button_style(
            layout.page,
            row,
            col,
            &ButtonStyle {
                text: digit.to_string(),
                size: "44".to_string(),
                color: 0xFFFFFF,
                bgcolor: 0x3B82F6, // Blue
            },
        )
        .await?;
    }

    // Row 2: Control buttons
    api.set_button_style(
        layout.page,
        2,
        0,
        &ButtonStyle {
            text: "⌫".to_string(),
            size: "44".to_string(),
            color: 0xFFFFFF,
            bgcolor: 0xEF4444, // Red
        },
    )
    .await?;

    api.set_button_style(
        layout.page,
        2,
        1,
        &ButtonStyle {
            text: "CLR".to_string(),
            size: "18".to_string(),
            color: 0xFFFFFF,
            bgcolor: 0xF59E0B, // Amber
        },
    )
    .await?;

    api.set_button_style(
        layout.page,
        2,
        2,
        &ButtonStyle {
            text: "↻".to_string(),
            size: "44".to_string(),
            color: 0xFFFFFF,
            bgcolor: 0x6B7280, // Gray
        },
    )
    .await?;

    api.set_button_style(
        layout.page,
        2,
        3,
        &ButtonStyle {
            text: "Filter".to_string(),
            size: "14".to_string(),
            color: 0xFFFFFF,
            bgcolor: 0x323232,
        },
    )
    .await?;

    api.set_button_style(
        layout.page,
        2,
        4,
        &ButtonStyle {
            text: "Files".to_string(),
            size: "14".to_string(),
            color: 0xFFFFFF,
            bgcolor: 0x323232,
        },
    )
    .await?;

    // Row 3: Slot buttons
    for slot in 0..5 {
        api.set_button_style(
            layout.page,
            3,
            slot,
            &ButtonStyle {
                text: format!("Slot {}", slot + 1),
                size: "14".to_string(),
                color: 0xFFFFFF,
                bgcolor: 0x009600, // Green
            },
        )
        .await?;
    }

    Ok(())
}

