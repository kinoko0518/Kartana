mod top_page;

use dioxus::prelude::*;
use top_page::Top;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Top {},
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
        Router::<Route> {}
    }
}
