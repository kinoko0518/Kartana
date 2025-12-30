use dioxus::prelude::*;

#[derive(PartialEq, Eq, Clone)]
struct Chapter {
    title: String,
    created_at: String,
}

#[derive(PartialEq, Eq, Clone)]
struct Series {
    title: String,
    is_favourite: bool,
    chapters: Vec<Chapter>,
}

const FAVOURITE_ICON: Asset = asset!("assets/icons/bookmark.svg");
const EDIT_ICON: Asset = asset!("assets/icons/edit.svg");
const READ_ICON: Asset = asset!("assets/icons/read.svg");

#[component]
fn DrawSeries(
    series: Series,
    on_favorite_click: EventHandler<MouseEvent>,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let fav_icon_style = if series.is_favourite {
        "background-color: #ffa0b0ff;"
    } else {
        "background-color: white;"
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
                div {
                    class: "action_icon",
                    style: "{fav_icon_style} mask-image: url({FAVOURITE_ICON}); -webkit-mask-image: url({FAVOURITE_ICON});",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_favorite_click.call(evt);
                    },
                }
            }
        }
    }
}

#[component]
pub fn Top() -> Element {
    let mut series: Signal<Vec<Series>> = use_signal(|| {
        vec![
            Series {
                title: "同志茜は高校生活を革命するそうです".into(),
                is_favourite: true,
                chapters: vec![
                    Chapter { title: "第1話".into(), created_at: "2024-01-01".into() },
                    Chapter { title: "第2話".into(), created_at: "2024-01-08".into() },
                ],
            },
            Series {
                title: "不欠望月、孰蟾宮主人乎？".into(),
                is_favourite: true,
                chapters: vec![
                     Chapter { title: "序章".into(), created_at: "2024-02-01".into() },
                ],
            }
        ]
    });

    let mut selected_series_index = use_signal(|| None::<usize>);

    rsx! {
        div {
            class: "top_layout",
            div {
                class: "series_grid",
                for (i, s) in series.read().clone().into_iter().enumerate() {
                    DrawSeries {
                        series: s,
                        on_favorite_click: move |_| {
                            let mut s = series.write();
                            s[i].is_favourite = !s[i].is_favourite;
                        },
                        on_click: move |_| {
                            selected_series_index.set(Some(i));
                        }
                    }
                }
            }
            if let Some(index) = selected_series_index() {
                div {
                    class: "chapter_list_panel",
                    h2 { "{series.read()[index].title}" }
                    ul {
                        for chapter in &series.read()[index].chapters {
                            li {
                                div {
                                    "{chapter.title}"
                                    br {}
                                    small { "{chapter.created_at}" }
                                }
                                div {
                                    class: "chapter_actions",
                                    div {
                                        class: "action_icon",
                                        style: "mask-image: url({EDIT_ICON}); -webkit-mask-image: url({EDIT_ICON});"
                                    }
                                    div {
                                        class: "action_icon",
                                        style: "mask-image: url({READ_ICON}); -webkit-mask-image: url({READ_ICON});"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
