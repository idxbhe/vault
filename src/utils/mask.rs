//! Content masking utilities for sensitive data

/// Default mask character
pub const MASK_CHAR: char = '•';

/// Masks sensitive content with bullet characters
pub fn mask_content(content: &str) -> String {
    content
        .chars()
        .map(|c| if c.is_whitespace() { c } else { MASK_CHAR })
        .collect()
}

/// Masks content but preserves length indication
pub fn mask_with_length(content: &str) -> String {
    let len = content.len();
    if len <= 8 {
        MASK_CHAR.to_string().repeat(len)
    } else {
        format!("{}... ({} chars)", MASK_CHAR.to_string().repeat(8), len)
    }
}

/// Partially reveals content (for password recovery)
/// `reveal_percentage` should be between 0.0 and 1.0
pub fn partial_reveal(content: &str, reveal_percentage: f32, seed: u64) -> String {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();

    if len == 0 {
        return String::new();
    }

    let reveal_count = ((len as f32) * reveal_percentage.clamp(0.0, 1.0)).ceil() as usize;

    // Use seeded RNG for consistent reveals per vault
    let mut rng = StdRng::seed_from_u64(seed);

    // Generate indices to reveal
    let mut indices: Vec<usize> = (0..len).collect();
    for i in (1..len).rev() {
        let j = rng.gen_range(0..=i);
        indices.swap(i, j);
    }
    let reveal_indices: std::collections::HashSet<usize> =
        indices.into_iter().take(reveal_count).collect();

    chars
        .iter()
        .enumerate()
        .map(|(i, c)| {
            if reveal_indices.contains(&i) {
                *c
            } else {
                MASK_CHAR
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_content() {
        assert_eq!(mask_content("secret"), "••••••");
        assert_eq!(mask_content(""), "");
    }

    #[test]
    fn test_mask_with_length() {
        assert_eq!(mask_with_length("short"), "•••••");
        assert_eq!(
            mask_with_length("this is a very long password"),
            "••••••••... (28 chars)"
        );
    }

    #[test]
    fn test_partial_reveal() {
        let content = "kucinghitam";
        let revealed = partial_reveal(content, 0.3, 12345);

        // Should have some revealed chars and some masked
        assert!(revealed.contains(MASK_CHAR));
        assert!(revealed.chars().any(|c| c != MASK_CHAR));

        // Same seed should produce same result
        let revealed2 = partial_reveal(content, 0.3, 12345);
        assert_eq!(revealed, revealed2);
    }
}
