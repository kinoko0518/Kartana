use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

const SERIES_PATH: &str = "data/series";

pub const FAVOURITE_ICON: Asset = asset!("assets/icons/bookmark.svg");
pub const EDIT_ICON: Asset = asset!("assets/icons/edit.svg");
pub const READ_ICON: Asset = asset!("assets/icons/read.svg");
pub const DELETE_ICON: Asset = asset!("assets/icons/delete.svg");

#[derive(PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Chapter {
    pub title: String,
    pub created_at: String,
}

#[derive(PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Series {
    pub title: String,
    pub is_favourite: bool,
    pub chapters: Vec<Chapter>,
}

impl Series {
    pub fn series_dir(title: &str) -> PathBuf {
        PathBuf::from(SERIES_PATH).join(title)
    }
    pub fn own_path(&self) -> PathBuf {
        Self::series_dir(&self.title)
    }
    pub fn save_series(&self) -> Result<(), Box<dyn std::error::Error>> {
        let series_dir = self.own_path();
        if !series_dir.exists() {
            fs::create_dir_all(&series_dir)?;
        }
        let meta_path = series_dir.join("series.toml");
        let mut file = File::create(&meta_path)?;
        writeln!(file, "{}", &toml::to_string(self).unwrap());
        Ok(())
    }
    pub fn delete_series(&self) -> Result<(), Box<dyn std::error::Error>> {
        let series_dir = self.own_path();
        if series_dir.exists() {
            fs::remove_dir_all(&series_dir)?;
        }
        Ok(())
    }
    pub fn load_series() -> Vec<Self> {
        let mut series_list = Vec::new();
        if let Ok(entries) = fs::read_dir(SERIES_PATH) {
            for entry in entries.flatten() {
                let path = entry.path();
                let series_toml = path.join("series.toml");
                if series_toml.exists() {
                    if let Ok(content) = fs::read_to_string(&series_toml) {
                        if let Ok(series) = toml::from_str::<Self>(&content) {
                            series_list.push(series);
                        }
                    }
                }
            }
        }
        series_list
    }
}

#[component]
pub fn ActionIcon(
    icon: Asset,
    onclick: EventHandler<MouseEvent>,
    class: Option<String>,
    style: Option<String>,
) -> Element {
    let extra_class = class.unwrap_or_default();
    let extra_style = style.unwrap_or_default();

    rsx! {
        div {
            class: "action_icon {extra_class}",
            style: "mask-image: url({icon}); -webkit-mask-image: url({icon}); {extra_style}",
            onclick: move |evt| {
                evt.stop_propagation();
                onclick.call(evt);
            },
        }
    }
}

#[component]
pub fn DrawSeries(
    series: Series,
    on_favorite_click: EventHandler<MouseEvent>,
    on_click: EventHandler<MouseEvent>,
    on_delete_click: EventHandler<MouseEvent>,
) -> Element {
    let fav_bg_color = if series.is_favourite {
        "#ffa0b0ff"
    } else {
        "white"
    };

    rsx! {
        div {
            class: "series_container",
            onclick: move |evt| on_click.call(evt),
            p {
                class: "series_title",
                "{series.title}"
            }
            div {
                class: "series_actions",
                ActionIcon {
                    icon: FAVOURITE_ICON,
                    style: "background-color: {fav_bg_color};",
                    onclick: move |evt| on_favorite_click.call(evt),
                }
                ActionIcon {
                    icon: DELETE_ICON,
                    class: "delete",
                    onclick: move |evt| on_delete_click.call(evt),
                }
            }
        }
    }
}
