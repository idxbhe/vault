use unicode_normalization::UnicodeNormalization;

fn main() {
    let inputs = vec!["İstanbul", "istanbul", "İSTANBUL", "ı", "I", "Москва", "москва", "ﬁle", "file"];
    
    for input in inputs {
        let normalized: String = input
            .nfd()
            .filter(|c| !('\u{0300}'..='\u{036f}').contains(c))
            .collect::<String>()
            .to_lowercase(); // Or case fold
            
        println!("{} -> {}", input, normalized);
    }
}
