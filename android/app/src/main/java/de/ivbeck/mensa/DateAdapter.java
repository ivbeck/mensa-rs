package de.ivbeck.mensa;

import android.content.Context;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;
import androidx.core.content.ContextCompat;
import androidx.recyclerview.widget.RecyclerView;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Calendar;
import java.util.List;
import java.util.Locale;

public final class DateAdapter extends RecyclerView.Adapter<DateAdapter.DateViewHolder> {

    public interface OnDateSelectedListener {
        void onDateSelected(Calendar date);
    }

    private final List<Calendar> dates = new ArrayList<>();
    private final LayoutInflater inflater;
    private final SimpleDateFormat weekdayFormat = new SimpleDateFormat("EEE", Locale.GERMAN);
    private final SimpleDateFormat dayFormat = new SimpleDateFormat("d", Locale.GERMAN);
    private Calendar selectedDate = Calendar.getInstance();
    private OnDateSelectedListener listener;

    public DateAdapter(Context context) {
        this.inflater = LayoutInflater.from(context);
    }

    public void setDates(List<Calendar> dates) {
        this.dates.clear();
        this.dates.addAll(dates);
        notifyDataSetChanged();
    }

    public void setSelectedDate(Calendar date) {
        Calendar oldSelected = this.selectedDate;
        this.selectedDate = date;
        for (int i = 0; i < dates.size(); i++) {
            if (isSameDay(dates.get(i), oldSelected)) {
                notifyItemChanged(i);
                break;
            }
        }
        for (int i = 0; i < dates.size(); i++) {
            if (isSameDay(dates.get(i), date)) {
                notifyItemChanged(i);
                break;
            }
        }
    }

    public void setOnDateSelectedListener(OnDateSelectedListener listener) {
        this.listener = listener;
    }

    private boolean isSameDay(Calendar a, Calendar b) {
        return a.get(Calendar.YEAR) == b.get(Calendar.YEAR)
            && a.get(Calendar.DAY_OF_YEAR) == b.get(Calendar.DAY_OF_YEAR);
    }

    @Override
    public DateViewHolder onCreateViewHolder(ViewGroup parent, int viewType) {
        View view = inflater.inflate(R.layout.item_date_chip, parent, false);
        return new DateViewHolder(view);
    }

    @Override
    public void onBindViewHolder(DateViewHolder holder, int position) {
        holder.bind(dates.get(position));
    }

    @Override
    public int getItemCount() {
        return dates.size();
    }

    final class DateViewHolder extends RecyclerView.ViewHolder {
        private final TextView weekday;
        private final TextView day;
        private final View indicator;

        DateViewHolder(View itemView) {
            super(itemView);
            weekday = itemView.findViewById(R.id.weekday);
            day = itemView.findViewById(R.id.day);
            indicator = itemView.findViewById(R.id.indicator);
        }

        void bind(Calendar date) {
            weekday.setText(weekdayFormat.format(date.getTime()));
            day.setText(dayFormat.format(date.getTime()));

            boolean isSelected = isSameDay(date, selectedDate);
            int context = isSelected ? R.color.ink : R.color.ink_faint;
            int dayColor = isSelected
                ? ContextCompat.getColor(itemView.getContext(), R.color.accent)
                : ContextCompat.getColor(itemView.getContext(), R.color.ink_faint);
            weekday.setTextColor(ContextCompat.getColor(itemView.getContext(), context));
            day.setTextColor(dayColor);
            indicator.setVisibility(isSelected ? View.VISIBLE : View.INVISIBLE);

            itemView.setOnClickListener(v -> {
                if (listener != null) listener.onDateSelected(date);
            });
        }
    }
}
