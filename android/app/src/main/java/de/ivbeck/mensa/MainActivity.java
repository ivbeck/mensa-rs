package de.ivbeck.mensa;

import android.app.Activity;
import android.graphics.Color;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.view.Gravity;
import android.view.View;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;

import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

import java.text.SimpleDateFormat;
import java.util.Calendar;
import java.util.Locale;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public final class MainActivity extends Activity {
    private static final int BG_DEEP = Color.rgb(0x10, 0x0E, 0x0B);
    private static final int BG_SURFACE = Color.rgb(0x1B, 0x17, 0x12);
    private static final int INK = Color.rgb(0xF1, 0xEA, 0xDD);
    private static final int INK_MUTED = Color.rgb(0x95, 0x8B, 0x7C);
    private static final int ACCENT = Color.rgb(0xE4, 0xA3, 0x3D);
    private static final int OXBLOOD_BG = Color.rgb(0x4A, 0x1C, 0x18);
    private static final int OXBLOOD_INK = Color.rgb(0xFF, 0xC9, 0xC0);
    private static final String LANG = "de";
    private static final String ALLERGENS = "Mi";
    private static final String FAVORITES = "";

    static {
        System.loadLibrary("mensa");
    }

    private final Calendar date = Calendar.getInstance();
    private final SimpleDateFormat apiDate = new SimpleDateFormat("yyyy-MM-dd", Locale.ROOT);
    private final SimpleDateFormat displayDate = new SimpleDateFormat("EEEE, dd.MM.yyyy", Locale.GERMANY);
    private final ExecutorService executor = Executors.newSingleThreadExecutor();
    private final Handler mainThread = new Handler(Looper.getMainLooper());
    private LinearLayout mealList;
    private TextView header;
    private Button filterButton;
    private boolean hideAllergens;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(buildContent());
        loadMenu();
    }

    @Override
    protected void onDestroy() {
        executor.shutdownNow();
        super.onDestroy();
    }

    private View buildContent() {
        LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setPadding(dp(20), dp(18), dp(20), dp(12));
        root.setBackgroundColor(BG_DEEP);

        header = label("Mensa am Schloss", 24, ACCENT);
        root.addView(header);

        LinearLayout toolbar = new LinearLayout(this);
        toolbar.setGravity(Gravity.CENTER_VERTICAL);
        toolbar.setPadding(0, dp(12), 0, dp(10));
        root.addView(toolbar);

        toolbar.addView(button("PREV", v -> {
            date.add(Calendar.DAY_OF_MONTH, -1);
            loadMenu();
        }));
        toolbar.addView(button("TODAY", v -> {
            date.setTimeInMillis(System.currentTimeMillis());
            loadMenu();
        }));
        toolbar.addView(button("NEXT", v -> {
            date.add(Calendar.DAY_OF_MONTH, 1);
            loadMenu();
        }));

        filterButton = button("FILTER", v -> {
            hideAllergens = !hideAllergens;
            filterButton.setText(hideAllergens ? "FILTER ON" : "FILTER");
            loadMenu();
        });
        toolbar.addView(filterButton);

        Button mlg = button("MLG MODE", v -> {
            header.setTextColor(Color.rgb(0x55, 0xFF, 0x88));
            mealList.setBackgroundColor(Color.rgb(0x18, 0x08, 0x24));
        });
        toolbar.addView(mlg);

        ScrollView scroll = new ScrollView(this);
        mealList = new LinearLayout(this);
        mealList.setOrientation(LinearLayout.VERTICAL);
        scroll.addView(mealList);
        root.addView(scroll, new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                0,
                1
        ));

        return root;
    }

    private void loadMenu() {
        String dateString = apiDate.format(date.getTime());
        header.setText("Mensa am Schloss - " + displayDate.format(date.getTime()));
        mealList.removeAllViews();
        mealList.addView(label("Wird angerichtet...", 16, INK_MUTED));

        executor.execute(() -> {
            String json = MenuBridge.fetchMenuJson(
                    dateString,
                    LANG,
                    ALLERGENS,
                    hideAllergens,
                    FAVORITES
            );
            mainThread.post(() -> renderMenu(json));
        });
    }

    private void renderMenu(String json) {
        mealList.removeAllViews();
        try {
            JSONObject response = new JSONObject(json);
            if (!response.optBoolean("ok")) {
                mealList.addView(label(response.optString("error", "Menu unavailable"), 16, OXBLOOD_INK));
                return;
            }

            JSONArray meals = response.getJSONArray("meals");
            if (meals.length() == 0) {
                mealList.addView(label("Keine Gerichte fuer dieses Datum.", 16, INK_MUTED));
                return;
            }

            for (int i = 0; i < meals.length(); i++) {
                mealList.addView(mealCard(meals.getJSONObject(i)));
            }
        } catch (JSONException error) {
            mealList.addView(label("Menu response could not be read: " + error.getMessage(), 16, OXBLOOD_INK));
        }
    }

    private View mealCard(JSONObject meal) throws JSONException {
        LinearLayout card = new LinearLayout(this);
        card.setOrientation(LinearLayout.VERTICAL);
        card.setPadding(dp(14), dp(12), dp(14), dp(12));
        card.setBackgroundColor(BG_SURFACE);

        String title = meal.getString("name") + " - " + meal.getString("price");
        if (meal.optBoolean("favorite")) {
            title = "* " + title;
        }
        card.addView(label(title, 18, INK));

        JSONArray items = meal.getJSONArray("items");
        for (int i = 0; i < items.length(); i++) {
            JSONObject item = items.getJSONObject(i);
            int color = item.optBoolean("has_allergen") ? OXBLOOD_INK : INK_MUTED;
            TextView itemView = label("- " + item.getString("text"), 14, color);
            if (item.optBoolean("has_allergen")) {
                itemView.setBackgroundColor(OXBLOOD_BG);
                itemView.setPadding(dp(8), dp(4), dp(8), dp(4));
            }
            card.addView(itemView);
        }

        LinearLayout.LayoutParams params = new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.WRAP_CONTENT
        );
        params.setMargins(0, 0, 0, dp(12));
        card.setLayoutParams(params);
        return card;
    }

    private TextView label(String text, int sp, int color) {
        TextView label = new TextView(this);
        label.setText(text);
        label.setTextColor(color);
        label.setTextSize(sp);
        label.setPadding(0, dp(4), 0, dp(4));
        return label;
    }

    private Button button(String text, View.OnClickListener listener) {
        Button button = new Button(this);
        button.setText(text);
        button.setTextColor(INK);
        button.setTextSize(11);
        button.setBackgroundColor(Color.TRANSPARENT);
        button.setOnClickListener(listener);
        return button;
    }

    private int dp(int value) {
        return Math.round(value * getResources().getDisplayMetrics().density);
    }
}
