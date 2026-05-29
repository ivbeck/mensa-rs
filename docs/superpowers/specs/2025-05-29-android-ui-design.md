# Android UI Modernization — Dark Minimal

**Date**: 2025-05-29
**Status**: Draft

## Overview

Modernize the Android Mensa app UI to a Dark Minimal design language. Retain the warm color palette while dramatically improving visual hierarchy, spacing, and component design.

## Design Language

### Color Palette
| Token | Hex | Usage |
|-------|-----|-------|
| BG_DEEP | `#0D0D0D` | Root background |
| BG_SURFACE | `#1A1A1A` | Card backgrounds |
| BG_ELEVATED | `#222222` | Elevated elements (badges, chips) |
| INK | `#FFFFFF` | Primary text |
| INK_MUTED | `#888888` | Secondary text, ingredients |
| INK_DIM | `#555555` | Tertiary, disabled states |
| ACCENT | `#E4A33D` | Primary accent — logo, selected dates, highlights |

### Typography
- **Font**: System default (Roboto on Android)
- **Sizes**: 11sp (chip/badge), 13sp (body/ingredients), 15sp (card title), 18sp (section header)
- **Weight**: 500 for titles, 400 for body text

### Spacing
- Base unit: 4dp
- Card padding: 16dp
- Card margin: 0 0 12dp 0
- Card border-radius: 16dp
- Card shadow: `0 4px 12px rgba(0,0,0,0.4)`

## Layout Structure

### Root Layout
- Full-screen `ScrollView` with `LinearLayout` as child
- Background: `BG_DEEP`
- Padding: 20dp horizontal, 18dp top, 12dp bottom

### Header Section
```
[Mensa am Schloss]          ← 18sp, ACCENT, bold
[Date: Monday, 26 May 2025] ← 13sp, INK_MUTED
```

### Date Strip (Navigation)
- Horizontal `RecyclerView` or `HorizontalScrollView` with `LinearLayout`
- Shows 7 days centered on selected date
- Each day: 48dp wide, 64dp tall chip
  - Weekday label: 11sp, INK_MUTED (unselected) / INK (selected)
  - Date number: 15sp, INK_MUTED (unselected) / ACCENT (selected)
- Selected chip: no background (text uses ACCENT)
- Unselected chip: no background (text uses INK_MUTED)

### Meal List
- Vertical `RecyclerView` with `LinearLayoutManager`
- Spacing: 12dp between cards

### Meal Card
- Background: `BG_SURFACE`
- Padding: 16dp all sides
- Border-radius: 16dp
- Shadow: subtle drop shadow
- Layout:
  ```
  [Meal Name                    ] [3.40€]   ← Row 1: title left, price badge right
  Hähnchenbrust · Soße · Reis                ← Row 2: ingredients, INK_MUTED, 12sp
  ```

### Price Badge
- Background: `BG_ELEVATED`
- Text: INK_MUTED, 11sp
- Padding: 4dp 10dp
- Border-radius: 6dp
- Position: top-right corner of card

### Allergen/Diet Tags (optional)
- Small pill chips below ingredients
- Background: semi-transparent colored bg
- Text: 9sp
- Example: `[Gluten]` in reddish tint

## Component Specs

### DateChip
- Width: 48dp, Height: 64dp
- Vertical layout: weekday (11sp) on top, date number (15sp) below
- Selected: date number in ACCENT color
- Unselected: all text in INK_MUTED
- No background/border

### MealCard
- Min-height: 72dp
- Title: 15sp, INK, weight 500
- Price badge: 11sp, INK_MUTED on BG_ELEVATED
- Ingredients: 12sp, INK_MUTED, separated by ` · `
- Card elevation via shadow (no border)

### Loading State
- Show skeleton cards (3 of them)
- Same dimensions as real cards
- Background: BG_SURFACE with animated shimmer

### Error State
- Centered error message in OXBLOOD color (#FF6B6B)
- Retry button below

### Empty State
- Centered message: "Keine Gerichte für dieses Datum."
- INK_MUTED color

## Navigation Interactions

- **Tap date chip**: Instantly loads that day's menu, scrolls date strip to center selected
- **Swipe left/right on meal list**: Optional gesture for prev/next day
- **Scroll date strip**: Standard horizontal scroll

## Implementation Notes

- Replace all `LinearLayout` based views with `RecyclerView`
- Use `CardView` or custom `MaterialCardView` for meal cards
- Implement custom `RecyclerView.ItemDecoration` for card spacing
- Use `ViewStub` or `setVisibility` for loading/error/empty states
- Maintain existing `MenuBridge.java` JNI bridge unchanged
- Keep `hideAllergens` filter toggle — move to a filter icon in header instead of separate button

## Filter Toggle

Move allergen filter from "FILTER" button to a filter icon (funnel) in the header area next to the title. Tapping toggles filter on/off. Visual indicator when active (accent color icon).

## Removed Elements

- "MLG MODE" button — remove entirely
- Old bordered buttons (PREV/TODAY/NEXT) — replaced by date strip
- All `Color.rgb()` hardcoded values — replace with named constants

## File Changes

- `MainActivity.java` — complete UI rewrite
- `styles.xml` — update theme colors to match palette
- New: `colors.xml` — define all color constants
- New: `dimens.xml` — define spacing and sizing constants
