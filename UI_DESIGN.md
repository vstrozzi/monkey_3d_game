# UI Design Documentation

## Overview

The UI has been redesigned with a modern, minimal aesthetic that enhances the player experience without cluttering the screen. The design focuses on clarity, visual hierarchy, and smooth readability.

## Design Philosophy

### Principles
1. **Minimal but Informative** - Show only what's necessary
2. **High Contrast** - Ensure text is always readable
3. **Visual Hierarchy** - Important information stands out
4. **Consistent Styling** - Cohesive look across all screens
5. **Themed Colors** - Color conveys meaning (red for target, green for success)

### Color Palette

```rust
// Dark overlay
Background: rgba(0, 0, 0, 0.85)

// Card container
Card Background: rgba(0.1, 0.1, 0.15, 0.95) - Dark blue-gray
Card Border: Accent color (contextual)

// Text colors
Primary Text: rgb(0.9, 0.9, 0.95) - Near white
Accent Text: Contextual (red/green)
Muted Text: rgba(1.0, 1.0, 1.0, 0.6) - Translucent white

// Accent colors
Red (Target): rgb(1.0, 0.3, 0.3)
Green (Success): rgb(0.3, 1.0, 0.5)
```

## UI States

### 1. Start Screen (NotStarted)

**Layout:**
- Full-screen semi-transparent overlay (85% opacity)
- Centered card with rounded corners and border
- Card contains:
  - Title with emoji: "🐵 PYRAMID SEEKER 🐵"
  - Horizontal divider
  - Game instructions
  - Control scheme
  - Start prompt

**Features:**
- Red accent color (matches target face)
- Large, readable text (48px title, 20px content)
- Emoji icons for visual interest
- Clear call-to-action

**Content:**
```
🐵 PYRAMID SEEKER 🐵
─────────────────────
Find the RED face of the pyramid!

🎮 Controls:
Arrow Keys / WASD - Rotate camera
SPACE - Check alignment

Press SPACE to start
```

### 2. Playing State (Playing)

**Layout:**
- Minimalist HUD in top-left corner
- Compact card with semi-transparent background
- Border with subtle glow
- Three information rows

**Features:**
- Non-intrusive placement
- Quick-glance information
- Icon-based visual hierarchy
- Dark background (70% opacity) for readability

**Content:**
```
┌──────────────────────────────┐
│ 🎯 Find the RED face         │
│ 📊 Attempts: 3               │
│ ⌨️ WASD/Arrows | SPACE: Check│
└──────────────────────────────┘
```

**Icons:**
- 🎯 Target/Goal
- 📊 Statistics
- ⌨️ Controls reminder

### 3. Win Screen (Won)

**Layout:**
- Full-screen semi-transparent overlay
- Centered victory card
- Dynamic title based on performance
- Statistics display
- Restart prompt

**Features:**
- Green accent color (success)
- Special title for perfect runs (1 attempt)
- Formatted statistics with icons
- Clear next action

**Titles:**
- Perfect run: "🏆 PERFECT! FIRST TRY! 🏆"
- Normal win: "✨ VICTORY! ✨"

**Content:**
```
✨ VICTORY! ✨
──────────────
You found the RED face!

⏱️  Time: 12.34s
🎯 Attempts: 3
📐 Accuracy: 95.2%

Press R to play again
```

## Technical Implementation

### Component Structure

#### Card Layout
```rust
Node {
    flex_direction: FlexDirection::Column,
    padding: UiRect::all(Val::Px(40.0)),
    row_gap: Val::Px(20.0),
    border: UiRect::all(Val::Px(3.0)),
    max_width: Val::Px(700.0),
}
```

#### Typography
- Title: 48px
- Content: 20px
- HUD: 18px (main), 16px (secondary), 14px (hints)

#### Spacing
- Card padding: 40px
- Row gap: 20px (cards), 10px (HUD)
- Border width: 3px (cards), 2px (HUD)

### Key Functions

#### `spawn_overlay()`
Creates a full-screen semi-transparent dark overlay.

**Usage:** Background for start and win screens.

#### `spawn_text_card()`
Creates a styled card with title, divider, and content.

**Parameters:**
- `title: &str` - Large heading text
- `content: &str` - Body text
- `accent_color: Color` - Border and title color

**Features:**
- Automatic centering
- Flexible content area
- Themed styling

## Before vs After

### Before
- Plain black backgrounds
- Basic white text
- No visual hierarchy
- Generic formatting
- Long text blocks

### After
- ✅ Semi-transparent overlays (better depth)
- ✅ Styled cards with borders
- ✅ Color-coded accents (red for target, green for success)
- ✅ Icon integration for visual communication
- ✅ Typography hierarchy (title, content, hints)
- ✅ Improved spacing and padding
- ✅ Contextual design (different feel per state)
- ✅ Performance feedback (special message for first-try wins)

## Accessibility Considerations

1. **High Contrast** - Dark backgrounds with bright text
2. **Icon + Text** - Information conveyed both ways
3. **Large Text** - Readable from typical viewing distance
4. **Clear Hierarchy** - Important info is visually prominent
5. **Consistent Layout** - Similar structures across states

## Future Enhancements

Possible improvements:
- Animated transitions between states
- Particle effects on win screen
- Sound effect integration points
- Difficulty selection UI
- Leaderboard/stats tracking
- Theme switcher (light/dark modes)
- Localization support (icon-heavy design helps)
- Accessibility options (font size, color blind modes)

## File Changes

**Modified:** `src/utils/game_functions.rs`
- Removed: `spawn_black_screen()`, `spawn_centered_text_black_screen()`
- Added: `spawn_overlay()`, `spawn_text_card()`
- Updated: All `game_ui()` match arms for new styling

## Testing Recommendations

1. Test all three states (start, playing, win)
2. Verify text readability at different resolutions
3. Check border rendering on various displays
4. Test emoji rendering (may vary by OS)
5. Verify color contrast ratios
6. Test with different attempt counts
7. Verify perfect run (1 attempt) special message

## Design Credits

- Emoji icons: Unicode standard
- Color scheme: Custom (blue-gray dark theme)
- Layout: Flexbox-based responsive design
- Inspired by: Modern game UI principles (minimalism, information density)