fn main() {
    let i_upper = "İ"; // Turkish I with dot
    let i_lower = i_upper.to_lowercase();
    println!("Upper: {}, Lower: {}", i_upper, i_lower);
    
    let ı_upper = "I"; // Turkish dotless I
    let ı_lower = ı_upper.to_lowercase();
    println!("Upper: {}, Lower: {}", ı_upper, ı_lower);

    let s = "A\u{0301}BC"; // A with acute accent (decomposed)
    let s_lower = s.to_lowercase();
    println!("Decomposed A with acute lower: {}", s_lower);
}
