mod top_page;
mod editor_page;

use dioxus::prelude::*;
use top_page::Top;
use editor_page::Editor;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Top {},
    #[route("/editor/:series_title/:chapter_title")]
    Editor { series_title: String, chapter_title: String },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const VARIABLES_CSS: Asset = asset!("/assets/css/variables.css");
const BASE_CSS: Asset = asset!("/assets/css/base.css");
const LAYOUT_CSS: Asset = asset!("/assets/css/layout.css");
const CARDS_CSS: Asset = asset!("/assets/css/cards.css");
const CHAPTER_LIST_CSS: Asset = asset!("/assets/css/chapter_list.css");
const FORMS_CSS: Asset = asset!("/assets/css/forms.css");
const ICONS_CSS: Asset = asset!("/assets/css/icons.css");
const MODAL_CSS: Asset = asset!("/assets/css/modal.css");
const EDITOR_CSS: Asset = asset!("/assets/css/editor.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: VARIABLES_CSS }
        document::Link { rel: "stylesheet", href: BASE_CSS }
        document::Link { rel: "stylesheet", href: LAYOUT_CSS }
        document::Link { rel: "stylesheet", href: CARDS_CSS }
        document::Link { rel: "stylesheet", href: CHAPTER_LIST_CSS }
        document::Link { rel: "stylesheet", href: FORMS_CSS }
        document::Link { rel: "stylesheet", href: ICONS_CSS }
        document::Link { rel: "stylesheet", href: MODAL_CSS }
        document::Link { rel: "stylesheet", href: EDITOR_CSS }
        Router::<Route> {}
    }
}
