fn fix_bracket_pairs_manual(text: &str) -> String {
    // Simplified version to test raw string parsing
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut result = String::new();
    
    while i < chars.len() {
        // Check for r#"..."#
        if chars[i] == 'r' && i + 1 < chars.len() && chars[i + 1] == '#' {
            let mut level = 0;
            let mut j = i + 1;
            while j < chars.len() && chars[j] == '#' {
                level += 1;
                j += 1;
            }
            if j < chars.len() && (chars[j] == '"' || chars[j] == '\'') {
                result.push_str(&format!("r{}{}...{}{}\"", "#".repeat(level), chars[j], chars[j], "#".repeat(level)));
            }
            break;
        }
        i += 1;
    }
    result
}

fn main() {
    let tests = vec![
        r#"r#"test"#"#,  // level 1
        r##"r##"test"##"##,  // level 2
        r###"r###"test"###"###,  // level 3
    ];
    
    for test in tests {
        println!("Input: {}", test);
        let result = fix_bracket_pairs_manual(test);
        println!("Parsed: {}", result);
        println!();
    }
}
