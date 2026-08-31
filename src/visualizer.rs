use crate::solver::validator::sanitize_letter;
use crate::solver::validator::UNKNOWN;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub fn init() {
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document");

    let listener_document = document.clone();
    let closure = Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(move |_event| {
        handle_input(&listener_document);
    }));
    document
        .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
        .expect("failed to add input listener");
    closure.forget();
}

fn handle_input(document: &web_sys::Document) {
    let correct = read_correct(document);
    let misplaced = read_misplaced(document);
    let excluded = read_excluded(document);

    web_sys::console::log_3(
        &format!("'{correct}'").into(),
        &format!("'{}'", misplaced.join("', '")).into(),
        &format!("'{excluded}'").into(),
    );
    let misplaced_refs: Vec<&str> = misplaced.iter().map(String::as_str).collect();

    match crate::solver::solve(&correct, &misplaced_refs, &excluded) {
        Ok(words) => {
            hide_error(document);
            render_results(document, &words);
        }
        Err(e) => {
            clear_results(document);
            show_error(document, &e.to_string());
            web_sys::console::error_1(&e.to_string().into());
        }
    }
}

fn show_error(document: &web_sys::Document, message: &str) {
    if let Some(el) = document.get_element_by_id("error") {
        el.set_text_content(Some(message));
        let _ = el.remove_attribute("hidden");
    }
}

fn hide_error(document: &web_sys::Document) {
    if let Some(el) = document.get_element_by_id("error") {
        el.set_text_content(None);
        let _ = el.set_attribute("hidden", "");
    }
}

fn render_results(document: &web_sys::Document, words: &[String]) {
    let Some(results) = document.get_element_by_id("results") else {
        return;
    };
    let html = words
        .iter()
        .map(|w| format!("<div>{}</div>", w))
        .collect::<String>();
    results.set_inner_html(&html);
}

fn clear_results(document: &web_sys::Document) {
    if let Some(results) = document.get_element_by_id("results") {
        results.set_inner_html("");
    }
}

fn read_correct(document: &web_sys::Document) -> String {
    let tiles = collect_tile_values(document.query_selector_all("#correct-letters .word-row input"));
    tiles_to_pattern(&tiles)
}

fn read_misplaced(document: &web_sys::Document) -> Vec<String> {
    let rows = document.query_selector_all("#misplaced-rows .word-row");
    let Ok(rows) = rows else {
        return Vec::new();
    };
    let mut patterns = Vec::new();
    for i in 0..rows.length() {
        if let Some(node) = rows.item(i) {
            if let Ok(row) = node.dyn_into::<web_sys::Element>() {
                let tiles = collect_tile_values(row.query_selector_all("input"));
                patterns.push(tiles_to_pattern(&tiles));
            }
        }
    }
    patterns
}

fn read_excluded(document: &web_sys::Document) -> String {
    if let Ok(Some(node)) = document.query_selector("#excluded-letters input") {
        if let Ok(input) = node.dyn_into::<web_sys::HtmlInputElement>() {
            return sanitize_excluded(&input.value());
        }
    }
    String::new()
}

fn collect_tile_values(node_list: Result<web_sys::NodeList, wasm_bindgen::JsValue>) -> Vec<String> {
    let Ok(node_list) = node_list else {
        return Vec::new();
    };
    let mut tiles = Vec::with_capacity(node_list.length() as usize);
    for i in 0..node_list.length() {
        if let Some(node) = node_list.item(i) {
            if let Ok(input) = node.dyn_into::<web_sys::HtmlInputElement>() {
                tiles.push(input.value());
            }
        }
    }
    tiles
}

fn sanitize_tile(value: &str) -> char {
    let c = value.chars().next().unwrap_or(UNKNOWN);
    sanitize_letter(c, true).unwrap_or(UNKNOWN)
}

fn sanitize_excluded(input: &str) -> String {
    input
        .chars()
        .filter_map(|c| sanitize_letter(c, false).ok())
        .collect()
}

pub fn tiles_to_pattern<T: AsRef<str>>(tiles: &[T]) -> String {
    tiles
        .iter()
        .map(|t| sanitize_tile(t.as_ref()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiles_to_pattern_all_empty() {
        assert_eq!(tiles_to_pattern(&["", "", "", "", ""]), "     ");
    }

    #[test]
    fn test_tiles_to_pattern_mixed() {
        assert_eq!(tiles_to_pattern(&["a", "", " ", "c", ""]), "a  c ");
    }

    #[test]
    fn test_sanitize_tile_lowercase() {
        assert_eq!(sanitize_tile("a"), 'a');
    }

    #[test]
    fn test_sanitize_tile_uppercase_lowered() {
        assert_eq!(sanitize_tile("A"), 'a');
    }

    #[test]
    fn test_sanitize_tile_empty_is_placeholder() {
        assert_eq!(sanitize_tile(""), ' ');
    }

    #[test]
    fn test_sanitize_tile_invalid_is_placeholder() {
        assert_eq!(sanitize_tile("."), ' ');
        assert_eq!(sanitize_tile("ñ"), ' ');
    }

    #[test]
    fn test_sanitize_excluded_lowercase() {
        assert_eq!(sanitize_excluded("abc"), "abc");
    }

    #[test]
    fn test_sanitize_excluded_uppercase_lowered() {
        assert_eq!(sanitize_excluded("ABC"), "abc");
    }

    #[test]
    fn test_sanitize_excluded_drops_invalid() {
        assert_eq!(sanitize_excluded("ab.c ñ"), "abc");
    }

    #[test]
    fn test_sanitize_excluded_empty() {
        assert_eq!(sanitize_excluded(""), "");
    }
}
