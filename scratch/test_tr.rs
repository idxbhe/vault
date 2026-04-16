fn main() {
    let a = "İstanbul";
    let b = "istanbul";
    println!("Standard Lower: {} == {}", a.to_lowercase(), b.to_lowercase());
    println!("Standard Eq: {}", a.to_lowercase() == b.to_lowercase());
}
