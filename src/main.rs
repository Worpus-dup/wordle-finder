mod solver;
mod words;

fn main() {
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document");
    let _body = document.body().expect("no body");
    // body.set_inner_html("<h1>Wordle Finder</h1>");
}
