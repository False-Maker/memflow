fn main() {
    let input = "print(\"hello (world\"";
    println!("Input: {}", input);
    println!("Input repr: {:?}", input);
    
    // Count brackets
    let open_count = input.matches('(').count();
    let close_count = input.matches(')').count();
    println!("Open: {}, Close: {}", open_count, close_count);
    
    // The first '(' is in "print(" - this needs closing
    // The second '(' is inside the string "(world" - should NOT close
    // So we should add 1 closing ')'
    let expected = "print(\"hello (world\")";
    println!("Expected: {}", expected);
}
