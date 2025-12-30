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
const DELETE_ICON: Asset = asset!("assets/icons/delete.svg");

#[component]
fn DrawSeries(
    series: Series,
    on_favorite_click: EventHandler<MouseEvent>,
    on_click: EventHandler<MouseEvent>,
    on_delete_click: EventHandler<MouseEvent>,
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
                div {
                    class: "action_icon delete",
                    style: "mask-image: url({DELETE_ICON}); -webkit-mask-image: url({DELETE_ICON});",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_delete_click.call(evt);
                    },
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
enum DeleteTarget {
    None,
    Series(usize),
    Chapter(usize, usize),
}

#[derive(Clone, PartialEq)]
enum PanelState {
    None,
    Selected(usize),
    CreateSeries,
    CreateChapter(usize),
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

    let mut panel_state = use_signal(|| PanelState::None);
    let mut delete_target = use_signal(|| DeleteTarget::None);
    let mut new_series_title = use_signal(|| String::new());
    let mut new_chapter_title = use_signal(|| String::new());

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
                            panel_state.set(PanelState::Selected(i));
                        },
                        on_delete_click: move |_| {
                            delete_target.set(DeleteTarget::Series(i));
                        }
                    }
                }
                // Create New Series Card
                div {
                    class: "series_container create_card",
                    onclick: move |_| {
                        panel_state.set(PanelState::CreateSeries);
                        new_series_title.set(String::new());
                    },
                    p { "+" }
                }
            }
            div {
                class: "chapter_list_panel",
                match panel_state() {
                    PanelState::Selected(index) => rsx! {
                        h2 { "{series.read()[index].title}" }
                        ul {
                            for (chapter_idx, chapter) in series.read()[index].chapters.clone().into_iter().enumerate() {
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
                                        div {
                                            class: "action_icon delete",
                                            style: "mask-image: url({DELETE_ICON}); -webkit-mask-image: url({DELETE_ICON});",
                                            onclick: move |_| {
                                                delete_target.set(DeleteTarget::Chapter(index, chapter_idx));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div {
                            class: "add_chapter_container",
                            button {
                                class: "add_chapter_button",
                                onclick: move |_| {
                                    panel_state.set(PanelState::CreateChapter(index));
                                    new_chapter_title.set(String::new());
                                },
                                "+"
                            }
                        }
                    },
                    PanelState::CreateSeries => rsx! {
                        div {
                            class: "create_form",
                            h2 { "新しいシリーズを作成" }
                            input {
                                value: "{new_series_title}",
                                oninput: move |evt| new_series_title.set(evt.value()),
                                placeholder: "シリーズタイトル",
                            }
                            div {
                                class: "form_actions",
                                button {
                                    onclick: move |_| {
                                        if !new_series_title().trim().is_empty() {
                                            series.write().push(Series {
                                                title: new_series_title(),
                                                is_favourite: false,
                                                chapters: vec![],
                                            });
                                            let new_index = series.read().len() - 1;
                                            panel_state.set(PanelState::Selected(new_index));
                                        }
                                    },
                                    "作成"
                                }
                                button {
                                    onclick: move |_| panel_state.set(PanelState::None),
                                    "キャンセル"
                                }
                            }
                        }
                    },
                    PanelState::CreateChapter(index) => rsx! {
                        div {
                            class: "create_form",
                            h2 { "新しいチャプターを作成" }
                             input {
                                value: "{new_chapter_title}",
                                oninput: move |evt| new_chapter_title.set(evt.value()),
                                placeholder: "チャプタータイトル",
                            }
                            div {
                                class: "form_actions",
                                button {
                                    onclick: move |_| {
                                        if !new_chapter_title().trim().is_empty() {
                                            series.write()[index].chapters.push(Chapter {
                                                title: new_chapter_title(),
                                                created_at: "2025-01-01".into(), // Default date for now
                                            });
                                            panel_state.set(PanelState::Selected(index));
                                        }
                                    },
                                    "作成"
                                }
                                button {
                                    onclick: move |_| panel_state.set(PanelState::Selected(index)),
                                    "キャンセル"
                                }
                            }
                        }
                    },
                    PanelState::None => rsx! {
                        p { "シリーズが選択されていません" }
                    },
                }
            }
        }
        if let DeleteTarget::Series(i) = delete_target() {
            div {
                class: "modal_overlay",
                div {
                    class: "modal_content",
                    p { "本当に「{series.read()[i].title}」を削除しますか？" }
                    div {
                        class: "form_actions",
                        style: "justify-content: center; gap: 20px; margin-top: 20px;",
                        button {
                            class: "danger",
                            onclick: move |_| {
                                series.write().remove(i);
                                match panel_state() {
                                    PanelState::Selected(selected_index) => {
                                        if selected_index == i {
                                            panel_state.set(PanelState::None);
                                        } else if selected_index > i {
                                            panel_state.set(PanelState::Selected(selected_index - 1));
                                        }
                                    }
                                    PanelState::CreateChapter(selected_index) => {
                                        if selected_index == i {
                                            panel_state.set(PanelState::None);
                                        } else if selected_index > i {
                                            panel_state.set(PanelState::CreateChapter(selected_index - 1));
                                        }
                                    }
                                    _ => {}
                                }
                                delete_target.set(DeleteTarget::None);
                            },
                            "はい"
                        }
                        button {
                            onclick: move |_| delete_target.set(DeleteTarget::None),
                            "いいえ"
                        }
                    }
                }
            }
        }
        if let DeleteTarget::Chapter(series_idx, chapter_idx) = delete_target() {
             div {
                class: "modal_overlay",
                div {
                    class: "modal_content",
                    p { "本当に「{series.read()[series_idx].chapters[chapter_idx].title}」を削除しますか？" }
                    div {
                        class: "form_actions",
                        style: "justify-content: center; gap: 20px; margin-top: 20px;",
                        button {
                            class: "danger",
                            onclick: move |_| {
                                series.write()[series_idx].chapters.remove(chapter_idx);
                                delete_target.set(DeleteTarget::None);
                            },
                            "はい"
                        }
                        button {
                            onclick: move |_| delete_target.set(DeleteTarget::None),
                            "いいえ"
                        }
                    }
                }
            }
        }
    }
}
