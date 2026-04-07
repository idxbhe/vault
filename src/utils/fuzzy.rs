//! Fuzzy search utilities wrapping nucleo

use nucleo::{Config, Matcher, Utf32Str};

/// Fuzzy matcher for search functionality
pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcher {
    /// Create a new fuzzy matcher with default configuration
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    /// Check if pattern matches the text and return the score
    /// Higher scores indicate better matches
    pub fn score(&mut self, pattern: &str, text: &str) -> Option<u16> {
        if pattern.is_empty() {
            return Some(0);
        }

        let mut pattern_buf = Vec::new();
        let mut text_buf = Vec::new();
        
        let pattern = Utf32Str::new(pattern, &mut pattern_buf);
        let text = Utf32Str::new(text, &mut text_buf);

        self.matcher.fuzzy_match(text, pattern)
    }

    /// Check if pattern matches the text (case-insensitive)
    pub fn matches(&mut self, pattern: &str, text: &str) -> bool {
        self.score(&pattern.to_lowercase(), &text.to_lowercase()).is_some()
    }
}

/// Search result with score for sorting
#[derive(Debug, Clone)]
pub struct SearchResult<T> {
    pub item: T,
    pub score: u16,
}

impl<T> SearchResult<T> {
    pub fn new(item: T, score: u16) -> Self {
        Self { item, score }
    }
}

/// Search through items and return sorted results
pub fn search<'a, T, F>(items: &'a [T], pattern: &str, get_text: F) -> Vec<SearchResult<&'a T>>
where
    F: Fn(&T) -> &str,
{
    if pattern.is_empty() {
        return items.iter().map(|item| SearchResult::new(item, 0)).collect();
    }

    let mut matcher = FuzzyMatcher::new();
    let pattern_lower = pattern.to_lowercase();
    
    let mut results: Vec<_> = items
        .iter()
        .filter_map(|item| {
            let text = get_text(item).to_lowercase();
            matcher.score(&pattern_lower, &text).map(|score| SearchResult::new(item, score))
        })
        .collect();

    // Sort by score descending
    results.sort_by(|a, b| b.score.cmp(&a.score));
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_matcher() {
        let mut matcher = FuzzyMatcher::new();
        
        assert!(matcher.matches("btc", "Bitcoin Wallet"));
        assert!(matcher.matches("wal", "Bitcoin Wallet"));
        assert!(!matcher.matches("xyz", "Bitcoin Wallet"));
    }

    #[test]
    fn test_search() {
        let items = vec!["Bitcoin Wallet", "Ethereum Keys", "Bank Password"];
        let results = search(&items, "bit", |s| s);
        
        assert!(!results.is_empty());
        assert_eq!(*results[0].item, "Bitcoin Wallet");
    }
}
