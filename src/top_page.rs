use dioxus::prelude::*;

const FAVOURITE_ICON: Asset = asset!("assets/icons/bookmark.svg");
const EDIT_ICON: Asset = asset!("assets/icons/edit.svg");
const READ_ICON: Asset = asset!("assets/icons/read.svg");
const DELETE_ICON: Asset = asset!("assets/icons/delete.svg");

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
fn ActionIcon(
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
fn CreateForm(
    title_header: String,
    placeholder: String,
    value: String,
    oninput: EventHandler<String>,
    oncreate: EventHandler<MouseEvent>,
    oncancel: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div {
            class: "create_form",
            h2 { "{title_header}" }
            input {
                value: "{value}",
                oninput: move |evt| oninput.call(evt.value()),
                placeholder: "{placeholder}",
            }
            div {
                class: "form_actions",
                button {
                    onclick: move |evt| oncreate.call(evt),
                    "作成"
                }
                button {
                    onclick: move |evt| oncancel.call(evt),
                    "キャンセル"
                }
            }
        }
    }
}

#[component]
fn ConfirmationModal(
    message: String,
    onconfirm: EventHandler<MouseEvent>,
    oncancel: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div {
            class: "modal_overlay",
            div {
                class: "modal_content",
                p { "{message}" }
                div {
                    class: "form_actions",
                    style: "justify-content: center; gap: 20px; margin-top: 20px;",
                    button {
                        class: "danger",
                        onclick: move |evt| onconfirm.call(evt),
                        "はい"
                    }
                    button {
                        onclick: move |evt| oncancel.call(evt),
                        "いいえ"
                    }
                }
            }
        }
    }
}

#[component]
fn DrawSeries(
    series: Series,
    on_favorite_click: EventHandler<MouseEvent>,
    on_click: EventHandler<MouseEvent>,
    on_delete_click: EventHandler<MouseEvent>,
) -> Element {
    let fav_bg_color = if series.is_favourite { "#ffa0b0ff" } else { "white" };

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
    let navigator = use_navigator();

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
                                        ActionIcon {
                                            icon: EDIT_ICON,
                                            onclick: move |_| {
                                                navigator.push(crate::Route::Editor {
                                                    series_title: series.read()[index].title.clone(),
                                                    chapter_title: series.read()[index].chapters[chapter_idx].title.clone(),
                                                });
                                            },
                                        }
                                        ActionIcon {
                                            icon: READ_ICON,
                                            onclick: |_| {}, // Placeholder
                                        }
                                        ActionIcon {
                                            icon: DELETE_ICON,
                                            class: "delete",
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
                        CreateForm {
                            title_header: "新しいシリーズを作成",
                            placeholder: "シリーズタイトル",
                            value: "{new_series_title}",
                            oninput: move |val: String| new_series_title.set(val),
                            oncreate: move |_| {
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
                            oncancel: move |_| panel_state.set(PanelState::None),
                        }
                    },
                    PanelState::CreateChapter(index) => rsx! {
                        CreateForm {
                            title_header: "新しいチャプターを作成",
                            placeholder: "チャプタータイトル",
                            value: "{new_chapter_title}",
                            oninput: move |val: String| new_chapter_title.set(val),
                            oncreate: move |_| {
                                if !new_chapter_title().trim().is_empty() {
                                    series.write()[index].chapters.push(Chapter {
                                        title: new_chapter_title(),
                                        created_at: "2025-01-01".into(),
                                    });
                                    panel_state.set(PanelState::Selected(index));
                                }
                            },
                            oncancel: move |_| panel_state.set(PanelState::Selected(index)),
                        }
                    },
                    PanelState::None => rsx! {
                        p { "シリーズが選択されていません" }
                    },
                }
            }
        }
        
        match delete_target() {
            DeleteTarget::Series(i) => rsx! {
                ConfirmationModal {
                    message: format!("本当に「{}」を削除しますか？", series.read()[i].title),
                    onconfirm: move |_| {
                        series.write().remove(i);
                        match panel_state() {
                            PanelState::Selected(selected_index) | PanelState::CreateChapter(selected_index) => {
                                if selected_index == i {
                                    panel_state.set(PanelState::None);
                                } else if selected_index > i {
                                    // If we are selecting a series after the one being deleted, its index shifts down by 1
                                    // We need to preserve the mode (Selected or CreateChapter)
                                    match panel_state() {
                                        PanelState::Selected(_) => panel_state.set(PanelState::Selected(selected_index - 1)),
                                        PanelState::CreateChapter(_) => panel_state.set(PanelState::CreateChapter(selected_index - 1)),
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                        delete_target.set(DeleteTarget::None);
                    },
                    oncancel: move |_| delete_target.set(DeleteTarget::None),
                }
            },
            DeleteTarget::Chapter(series_idx, chapter_idx) => rsx! {
                ConfirmationModal {
                    message: format!("本当に「{}」を削除しますか？", series.read()[series_idx].chapters[chapter_idx].title),
                    onconfirm: move |_| {
                        series.write()[series_idx].chapters.remove(chapter_idx);
                        delete_target.set(DeleteTarget::None);
                    },
                    oncancel: move |_| delete_target.set(DeleteTarget::None),
                }
            },
            DeleteTarget::None => rsx! {},
        }
    }
}
