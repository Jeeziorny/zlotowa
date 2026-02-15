import TotalStats from "./TotalStats.svelte";
import SpendingByCategory from "./SpendingByCategory.svelte";
import BiggestExpense from "./BiggestExpense.svelte";
import MonthlyTrend from "./MonthlyTrend.svelte";
import MostFrequent from "./MostFrequent.svelte";

/**
 * Widget registry.
 *
 * To add a new widget:
 * 1. Create a Svelte component in src/lib/widgets/ that accepts an `expenses` prop
 * 2. Import it here and add an entry to this array
 *
 * Each widget component receives:
 *   - expenses: Array<{ id, title, amount, date, category, classification_source }>
 */
export const widgets = [
  {
    id: "total-stats",
    name: "Total Stats",
    description: "Summary cards showing total expenses, transaction count, and category count.",
    size: "full", // "full" = spans entire row, "half" = takes one grid cell
    component: TotalStats,
  },
  {
    id: "spending-by-category",
    name: "Spending by Category",
    description: "Bar breakdown of spending per category.",
    size: "half",
    component: SpendingByCategory,
  },
  {
    id: "biggest-expense",
    name: "Biggest Expense",
    description: "Shows the single largest expense this month.",
    size: "half",
    component: BiggestExpense,
  },
  {
    id: "monthly-trend",
    name: "Monthly Trend",
    description: "Total spending per month as a simple bar chart.",
    size: "half",
    component: MonthlyTrend,
  },
  {
    id: "most-frequent",
    name: "Most Frequent",
    description: "Top 5 most frequently occurring expense titles.",
    size: "half",
    component: MostFrequent,
  },
];

/** Default widget IDs shown on a fresh dashboard. */
export const defaultWidgetIds = [
  "total-stats",
  "spending-by-category",
  "biggest-expense",
];
