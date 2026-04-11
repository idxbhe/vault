1. Add `ItemKind::Totp` to `KindSelectorState` in `src/ui/widgets/kind_selector.rs`.
   - Update the `Default` implementation for `KindSelectorState` to include `ItemKind::Totp`.
   - Check if the maximum `selected` boundary in the `test_selector_navigation` test logic changes. Yes, from 5 to 6.
2. Verify with tests and pre-commit steps.
