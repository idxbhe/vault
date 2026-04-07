# Testing Item Save Flow

## Issue Fixed
**Problem**: Items created in vault were not being saved to disk automatically. When user pressed Enter to save, item was only stored in memory. If app crashed or user pressed Esc/Ctrl+Q, changes were lost and vault could become corrupted.

**Solution**: Auto-save vault to disk immediately after successful item creation/edit via FormSubmit.

## Testing Steps

### Test 1: Create New Item and Verify Persistence

1. **Start the app**:
   ```bash
   cargo run --release
   ```

2. **Unlock test2 vault**:
   - Select `test2` vault (use j/k or arrow keys)
   - Press Enter
   - Enter password: `sudounlock`
   - Press Enter

3. **Create a new Seed Phrase item**:
   - Press `n` to create new item
   - Select `Seed Phrase` (should be highlighted by default)
   - Press Enter
   - Fill in the form:
     - **Title**: `My Test Seed`
     - **Seed phrase**: `test word1 word2 word3 word4 word5 word6 word7 word8 word9 word10 word11 word12`
     - **Notes** (optional): `Test note`
   - Navigate fields with Tab/Shift+Tab
   - Press **Enter** to save
   - You should see notification: **"Item created and saved"**

4. **Verify item appears in list**:
   - Item should appear in the left pane
   - Select it and verify details show in right pane

5. **Exit app normally**:
   - Press `q` to quit
   - Should exit cleanly (no "unsaved changes" warning)

6. **Restart and verify persistence**:
   ```bash
   cargo run --release
   ```
   - Unlock test2 vault again (password: `sudounlock`)
   - **VERIFY**: Your "My Test Seed" item should still be there
   - Select it and verify all fields match what you entered

### Test 2: Test Esc Behavior (No Corruption)

1. **Create a new item**:
   - Press `n`, select any type, press Enter
   - Fill in some fields

2. **Press Esc to cancel**:
   - Press `Esc`
   - Form should close
   - Item should NOT be created (check left pane)

3. **Verify no corruption**:
   - Press `q` to quit
   - No warning should appear
   - Restart app and unlock vault
   - Vault should open normally without errors

### Test 3: Edit Existing Item

1. **Select an existing item**

2. **Press `e` to edit**

3. **Modify some fields**:
   - Change title or content
   - Press **Enter** to save
   - Should see: **"Item updated and saved"**

4. **Exit and restart**:
   - Press `q`
   - Restart app and unlock vault
   - **VERIFY**: Changes are persisted

### Test 4: Ctrl+Q Force Quit (Auto-saved)

1. **Create a new item** and save with Enter
   - Should see "Item created and saved"

2. **Press Ctrl+Q** immediately
   - App should force quit

3. **Restart and unlock**:
   - **VERIFY**: Item IS saved (because auto-save on FormSubmit)

## Expected Behavior

✅ **Enter on form**: Auto-saves to disk immediately
✅ **Esc on form**: Closes form, discards unsaved changes, no corruption
✅ **q quit**: Exits cleanly if no pending changes
✅ **Ctrl+Q**: Force quits, but items saved via Enter are already on disk

## What Was Fixed

### Before:
- `FormSubmit` only marked vault as dirty, didn't save to disk
- User had to manually press save (but no save keybinding was obvious)
- Esc closed form without warning, could lose data
- Ctrl+Q would discard unsaved changes

### After:
- `FormSubmit` immediately saves to disk via `WriteVaultFile` effect
- `is_dirty` flag is cleared after auto-save
- Esc safely closes form (changes only saved if user pressed Enter)
- Ctrl+Q is safe because Enter already saved

## Files Modified

1. **src/app/update.rs** (Line 767-835):
   - `FormSubmit`: Added auto-save via `WriteVaultFile` effect
   - Returns `Effect::WriteVaultFile` for both NewItem and EditItem
   - Sets `is_dirty = false` before save
   - Updated notification text to "created and saved" / "updated and saved"

2. **src/app/update.rs** (Line 740-752):
   - `InputCancel`: Now safely closes forms without corruption
   - Only clears input buffer for non-form contexts

## Verification Command

```bash
# Clean test
rm -f test2.vault
cargo run --release
# Create test2 vault (password: sudounlock)
# Create item, save with Enter, quit with q
# Restart and verify item persists
```

## Success Criteria

- ✅ Items persist after app restart
- ✅ No "unsaved changes" warning after saving with Enter
- ✅ Esc closes form safely
- ✅ No vault corruption
- ✅ All 138 tests passing
