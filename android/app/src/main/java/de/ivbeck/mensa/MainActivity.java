package de.ivbeck.mensa;

import android.app.Activity;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.view.View;
import android.widget.ImageButton;
import android.widget.LinearLayout;
import android.widget.TextView;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Calendar;
import java.util.List;
import java.util.Locale;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public final class MainActivity extends Activity {
    private static final String LANG = "de";
    private static final String ALLERGENS = "Mi";
    private static final String FAVORITES = "";

    static {
        System.loadLibrary("mensa");
    }

    private final Calendar selectedDate = Calendar.getInstance();
    private final SimpleDateFormat apiDate = new SimpleDateFormat("yyyy-MM-dd", Locale.ROOT);
    private final SimpleDateFormat displayDate = new SimpleDateFormat("EEEE, dd.MM.yyyy", Locale.GERMANY);
    private final ExecutorService executor = Executors.newSingleThreadExecutor();
    private final Handler mainThread = new Handler(Looper.getMainLooper());

    private RecyclerView dateStrip;
    private DateAdapter dateAdapter;
    private RecyclerView mealList;
    private MealAdapter mealAdapter;
    private TextView header;
    private TextView subheader;
    private ImageButton filterButton;
    private boolean hideAllergens;
    private View loadingView;
    private View errorView;
    private TextView errorMessage;
    private View emptyView;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        initViews();
        setupDateStrip();
        setupMealList();
        loadMenu();
    }

    @Override
    protected void onDestroy() {
        executor.shutdownNow();
        super.onDestroy();
    }

    private void initViews() {
        header = findViewById(R.id.header);
        subheader = findViewById(R.id.subheader);
        dateStrip = findViewById(R.id.date_strip);
        mealList = findViewById(R.id.meal_list);
        filterButton = findViewById(R.id.filter_button);
        loadingView = findViewById(R.id.loading_view);
        errorView = findViewById(R.id.error_view);
        errorMessage = findViewById(R.id.error_message);
        emptyView = findViewById(R.id.empty_view);

        filterButton.setOnClickListener(v -> {
            hideAllergens = !hideAllergens;
            filterButton.setAlpha(hideAllergens ? 1.0f : 0.5f);
            loadMenu();
        });
        filterButton.setAlpha(hideAllergens ? 1.0f : 0.5f);
    }

    private void setupDateStrip() {
        dateAdapter = new DateAdapter(this);
        dateStrip.setLayoutManager(
            new LinearLayoutManager(this, LinearLayoutManager.HORIZONTAL, false));
        dateStrip.setAdapter(dateAdapter);
        dateAdapter.setOnDateSelectedListener(date -> {
            selectedDate.setTimeInMillis(date.getTimeInMillis());
            dateAdapter.setSelectedDate(selectedDate);
            loadMenu();
        });
        populateDateStrip();
    }

    private void populateDateStrip() {
        List<Calendar> dates = new ArrayList<>();
        Calendar cal = (Calendar) selectedDate.clone();
        cal.add(Calendar.DAY_OF_MONTH, -3);
        for (int i = 0; i < 7; i++) {
            Calendar c = (Calendar) cal.clone();
            dates.add(c);
            cal.add(Calendar.DAY_OF_MONTH, 1);
        }
        dateAdapter.setDates(dates);
        dateAdapter.setSelectedDate(selectedDate);
    }

    private void setupMealList() {
        mealAdapter = new MealAdapter(this);
        mealList.setLayoutManager(new LinearLayoutManager(this));
        mealList.setAdapter(mealAdapter);
    }

    private void loadMenu() {
        String dateString = apiDate.format(selectedDate.getTime());
        header.setText("Mensa am Schloss");
        subheader.setText(displayDate.format(selectedDate.getTime()));

        showLoading();

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

    private void showLoading() {
        loadingView.setVisibility(View.VISIBLE);
        errorView.setVisibility(View.GONE);
        emptyView.setVisibility(View.GONE);
    }

    private void renderMenu(String json) {
        loadingView.setVisibility(View.GONE);
        errorView.setVisibility(View.GONE);
        emptyView.setVisibility(View.GONE);

        try {
            JSONObject response = new JSONObject(json);
            if (!response.optBoolean("ok")) {
                showError(response.optString("error", "Menu unavailable"));
                return;
            }

            JSONArray meals = response.getJSONArray("meals");
            if (meals.length() == 0) {
                emptyView.setVisibility(View.VISIBLE);
                return;
            }

            mealAdapter.setMeals(meals);
        } catch (JSONException error) {
            showError("Menu response could not be read: " + error.getMessage());
        }
    }

    private void showError(String message) {
        errorView.setVisibility(View.VISIBLE);
        errorMessage.setText(message);
        errorView.findViewById(R.id.retry_button).setOnClickListener(v -> loadMenu());
    }
}
