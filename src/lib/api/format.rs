pub fn escape_md(text: &str) -> String {
    let escapes = vec![
        '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
    ];

    let mut result = String::new();

    for c in text.chars() {
        if escapes.contains(&c) {
            result.push('\\');
        }
        result.push(c);
    }

    result
}

pub fn escape_code(text: &str) -> String {
    let escapes = vec!['`', '\\'];

    let mut result = String::new();

    for c in text.chars() {
        if escapes.contains(&c) {
            result.push('\\');
        }
        result.push(c);
    }

    result
}
