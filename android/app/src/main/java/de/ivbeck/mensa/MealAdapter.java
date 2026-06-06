package de.ivbeck.mensa;

import android.content.Context;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;
import androidx.recyclerview.widget.RecyclerView;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;
import java.util.ArrayList;
import java.util.List;

public final class MealAdapter extends RecyclerView.Adapter<MealAdapter.MealViewHolder> {

    public interface OnMealClickListener {
        void onMealClick(JSONObject meal);
    }

    private final List<JSONObject> meals = new ArrayList<>();
    private final LayoutInflater inflater;
    private OnMealClickListener listener;

    public MealAdapter(Context context) {
        this.inflater = LayoutInflater.from(context);
    }

    public void setMeals(JSONArray mealsArray) throws JSONException {
        meals.clear();
        for (int i = 0; i < mealsArray.length(); i++) {
            meals.add(mealsArray.getJSONObject(i));
        }
        notifyDataSetChanged();
    }

    public void setOnMealClickListener(OnMealClickListener listener) {
        this.listener = listener;
    }

    @Override
    public MealViewHolder onCreateViewHolder(ViewGroup parent, int viewType) {
        View view = inflater.inflate(R.layout.item_meal_card, parent, false);
        return new MealViewHolder(view);
    }

    @Override
    public void onBindViewHolder(MealViewHolder holder, int position) {
        try {
            holder.bind(meals.get(position));
        } catch (JSONException e) {
            holder.bindError(e.getMessage());
        }
    }

    @Override
    public int getItemCount() {
        return meals.size();
    }

    final class MealViewHolder extends RecyclerView.ViewHolder {
        private final TextView index;
        private final TextView title;
        private final TextView priceBadge;
        private final TextView ingredients;
        private final View card;

        MealViewHolder(View itemView) {
            super(itemView);
            card = itemView.findViewById(R.id.card);
            index = itemView.findViewById(R.id.index);
            title = itemView.findViewById(R.id.meal_title);
            priceBadge = itemView.findViewById(R.id.price_badge);
            ingredients = itemView.findViewById(R.id.meal_ingredients);
        }

        void bind(JSONObject meal) throws JSONException {
            index.setText(String.format(java.util.Locale.ROOT, "%02d", getBindingAdapterPosition() + 1));

            String name = meal.getString("name");
            if (meal.optBoolean("favorite")) {
                name = "★ " + name;
            }
            title.setText(name);
            priceBadge.setText(meal.getString("price"));

            StringBuilder sb = new StringBuilder();
            JSONArray items = meal.getJSONArray("items");
            for (int i = 0; i < items.length(); i++) {
                if (i > 0) sb.append(" · ");
                sb.append(items.getJSONObject(i).getString("text"));
            }
            ingredients.setText(sb.toString());

            card.setOnClickListener(v -> {
                if (listener != null) listener.onMealClick(meal);
            });
        }

        void bindError(String error) {
            title.setText("Error loading meal");
            ingredients.setText(error);
            priceBadge.setVisibility(View.GONE);
        }
    }
}
