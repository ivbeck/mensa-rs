package de.ivbeck.mensa;

final class MenuBridge {
    private MenuBridge() {
    }

    static native String fetchMenuJson(
            String date,
            String lang,
            String allergens,
            boolean hideAllergens,
            String favorites
    );
}
