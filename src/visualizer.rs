use crate::solver::error::SolverError;
use crate::solver::validator::sanitize_letter;
use crate::solver::validator::UNKNOWN;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub fn init() {
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document");

    let input_doc = document.clone();
    let input_closure = Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(move |event| {
        let _ = handle_input(&input_doc, &event);
    }));
    document
        .add_event_listener_with_callback("input", input_closure.as_ref().unchecked_ref())
        .expect("failed to add input listener");
    // Intentionally leaks the closure for the lifetime of the page. The WASM
    // runtime needs this closure to outlive `init()` so the browser can keep
    // invoking it as the event listener; dropping it here would invalidate the
    // listener. This leak is required by the wasm-bindgen `Closure` contract.
    input_closure.forget();

    let key_doc = document.clone();
    let key_closure = Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(move |event| {
        let _ = backspace_navigate(&key_doc, &event);
    }));
    document
        .add_event_listener_with_callback("keydown", key_closure.as_ref().unchecked_ref())
        .expect("failed to add keydown listener");
    key_closure.forget();

    let click_doc = document.clone();
    let click_closure = Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(move |event| {
        let _ = handle_click(&click_doc, &event);
    }));
    document
        .add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())
        .expect("failed to add click listener");
    click_closure.forget();
}

fn handle_click(document: &web_sys::Document, event: &web_sys::Event) -> Option<()> {
    let target = event.target()?;
    let target = target.dyn_into::<web_sys::Element>().ok()?;

    if target.id() == "add-row" {
        add_row(document);
    } else if target.class_list().contains("remove-row") {
        if let Ok(Some(row)) = target.closest(".word-row") {
            remove_row(document, &row);
        }
    } else if target.id() == "clear-all" {
        clear_all(document);
    }
    Some(())
}

fn add_row(document: &web_sys::Document) -> Option<()> {
    let rows = document.get_element_by_id("misplaced-rows")?;
    if rows.children().length() >= 5 {
        return None;
    }
    let first = rows.first_element_child()?;
    let clone = first.clone_node_with_deep(true).ok()?;
    let clone_el = clone.dyn_into::<web_sys::Element>().ok()?;
    let _ = clear_row_inputs(&clone_el);
    let _ = rows.append_child(&clone_el);
    Some(())
}

fn remove_row(document: &web_sys::Document, row: &web_sys::Element) -> Option<()> {
    let rows = document.get_element_by_id("misplaced-rows")?;
    if rows.children().length() > 1 {
        row.remove();
    } else {
        let _ = clear_row_inputs(row);
    }
    let _ = refresh_results(document);
    Some(())
}

fn clear_all(document: &web_sys::Document) {
    for input in get_tile_inputs(document) {
        input.set_value("");
    }
    if let Ok(Some(node)) = document.query_selector("#excluded-letters input")
        && let Ok(input) = node.dyn_into::<web_sys::HtmlInputElement>()
    {
        input.set_value("");
    }
    if let Some(error) = document.get_element_by_id("error") {
        error.set_text_content(None);
        let _ = error.set_attribute("hidden", "");
    }
    if let Some(results) = document.get_element_by_id("results") {
        results.set_inner_html("");
    }
}

fn clear_row_inputs(row: &web_sys::Element) -> Option<()> {
    let inputs = row.query_selector_all("input").ok()?;
    for i in 0..inputs.length() {
        if let Some(node) = inputs.item(i)
            && let Ok(input) = node.dyn_into::<web_sys::HtmlInputElement>()
        {
            input.set_value("");
        }
    }
    Some(())
}

fn get_tile_inputs(document: &web_sys::Document) -> Vec<web_sys::HtmlInputElement> {
    let mut inputs = Vec::new();
    if let Ok(list) = document.query_selector_all(".word-row input") {
        for i in 0..list.length() {
            if let Some(node) = list.item(i)
                && let Ok(input) = node.dyn_into::<web_sys::HtmlInputElement>()
            {
                inputs.push(input);
            }
        }
    }
    inputs
}

fn focus_next(document: &web_sys::Document, target: &web_sys::HtmlInputElement) -> Option<()> {
    let inputs = get_tile_inputs(document);
    let index = inputs.iter().position(|i| i == target)?;
    if index + 1 < inputs.len() {
        let _ = inputs[index + 1].focus();
    }
    Some(())
}

fn backspace_navigate(document: &web_sys::Document, event: &web_sys::Event) -> Option<()> {
    let keyboard = event.dyn_ref::<web_sys::KeyboardEvent>()?;
    if keyboard.key() != "Backspace" {
        return None;
    }
    let target = event.target()?;
    let target = target.dyn_into::<web_sys::HtmlInputElement>().ok()?;
    let inputs = get_tile_inputs(document);
    let index = inputs.iter().position(|i| i == &target)?;
    if !target.value().is_empty() {
        return None;
    }
    if index > 0 {
        let _ = inputs[index - 1].focus();
    }
    Some(())
}

fn handle_input(document: &web_sys::Document, event: &web_sys::Event) -> Option<()> {
    let target = event.target()?;
    let target = target.dyn_into::<web_sys::HtmlInputElement>().ok()?;

    if target
        .unchecked_ref::<web_sys::Element>()
        .closest(".word-row")
        .map(|el| el.is_none())
        .unwrap_or(true)
    {
        let raw = target.value();
        let sanitized = sanitize_excluded(&raw);
        if sanitized != raw {
            target.set_value(&sanitized);
        }
        refresh_results(document);
        if let Some(c) = first_dropped_char(&raw)
            && !is_error_visible(document)
        {
            show_error(document, &SolverError::InvalidCharacter(c).to_string());
        }
        // We are finished here
        return Some(());
    }

    match classify_tile(&target.value()) {
        TileValue::Valid(ch) => {
            target.set_value(&ch.to_string());
            let _ = focus_next(document, &target);
            refresh_results(document)
        }
        TileValue::Space => {
            target.set_value("");
            let _ = focus_next(document, &target);
            refresh_results(document)
        }
        TileValue::Empty => refresh_results(document),
        TileValue::Invalid(c) => {
            target.set_value("");
            show_error(document, &SolverError::InvalidCharacter(c).to_string());
            Some(())
        }
    }
}

fn refresh_results(document: &web_sys::Document) -> Option<()> {
    let correct = read_correct(document);
    let misplaced = read_misplaced(document);
    let excluded = read_excluded(document);

    #[cfg(debug_assertions)]
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
    Some(())
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

fn is_error_visible(document: &web_sys::Document) -> bool {
    document
        .get_element_by_id("error")
        .map(|el| !el.has_attribute("hidden"))
        .unwrap_or(false)
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
        if let Some(node) = rows.item(i)
            && let Ok(row) = node.dyn_into::<web_sys::Element>()
        {
            let tiles = collect_tile_values(row.query_selector_all("input"));
            patterns.push(tiles_to_pattern(&tiles));
        }
    }
    patterns
}

fn read_excluded(document: &web_sys::Document) -> String {
    if let Ok(Some(node)) = document.query_selector("#excluded-letters input")
        && let Ok(input) = node.dyn_into::<web_sys::HtmlInputElement>()
    {
        return sanitize_excluded(&input.value());
    }
    String::new()
}

fn collect_tile_values(node_list: Result<web_sys::NodeList, wasm_bindgen::JsValue>) -> Vec<String> {
    let Ok(node_list) = node_list else {
        return Vec::new();
    };
    let mut tiles = Vec::with_capacity(node_list.length() as usize);
    for i in 0..node_list.length() {
        if let Some(node) = node_list.item(i)
            && let Ok(input) = node.dyn_into::<web_sys::HtmlInputElement>()
        {
            tiles.push(input.value());
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

fn first_dropped_char(input: &str) -> Option<char> {
    input.chars().find(|c| sanitize_letter(*c, false).is_err())
}

pub fn tiles_to_pattern<T: AsRef<str>>(tiles: &[T]) -> String {
    tiles
        .iter()
        .map(|t| sanitize_tile(t.as_ref()))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TileValue {
    Empty,
    Space,
    Valid(char),
    Invalid(char),
}

fn classify_tile(value: &str) -> TileValue {
    match value.chars().next() {
        None => TileValue::Empty,
        Some(c) if c == UNKNOWN => TileValue::Space,
        Some(c) => match sanitize_letter(c, false) {
            Ok(ch) => TileValue::Valid(ch),
            Err(_) => TileValue::Invalid(c),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! classify_tile_case {
        ($name:ident, $input:expr => $expected:expr) => {
            #[test]
            fn $name() {
                assert_eq!(classify_tile($input), $expected);
            }
        };
    }

    classify_tile_case!(test_classify_tile_empty, "" => TileValue::Empty);
    classify_tile_case!(test_classify_tile_space, " " => TileValue::Space);
    classify_tile_case!(test_classify_tile_valid, "a" => TileValue::Valid('a'));
    classify_tile_case!(test_classify_tile_uppercase_lowered, "A" => TileValue::Valid('a'));
    classify_tile_case!(test_classify_tile_symbol, "." => TileValue::Invalid('.'));
    classify_tile_case!(test_classify_tile_non_ascii, "ñ" => TileValue::Invalid('ñ'));
    classify_tile_case!(test_classify_tile_digit, "1" => TileValue::Invalid('1'));
    classify_tile_case!(test_classify_tile_paste_first_valid, "ab" => TileValue::Valid('a'));
    classify_tile_case!(test_classify_tile_paste_first_space, " a" => TileValue::Space);
    classify_tile_case!(test_classify_tile_paste_first_invalid, "!x" => TileValue::Invalid('!'));

    macro_rules! first_dropped_char_case {
        ($name:ident, $input:expr => $expected:expr) => {
            #[test]
            fn $name() {
                assert_eq!(first_dropped_char($input), $expected);
            }
        };
    }

    first_dropped_char_case!(test_first_dropped_char_none, "abc" => None);
    first_dropped_char_case!(test_first_dropped_char_uppercase_no_drop, "ABC" => None);
    first_dropped_char_case!(test_first_dropped_char_empty, "" => None);
    first_dropped_char_case!(test_first_dropped_char_dot, "ab.c" => Some('.'));
    first_dropped_char_case!(test_first_dropped_char_space, "a b" => Some(' '));
    first_dropped_char_case!(test_first_dropped_char_non_ascii, "ñbc" => Some('ñ'));
    first_dropped_char_case!(test_first_dropped_char_first_wins, "ab.c ñ" => Some('.'));

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
