import TotalStats from "./TotalStats.svelte";
import SpendingByCategory from "./SpendingByCategory.svelte";
import BiggestExpense from "./BiggestExpense.svelte";
import MonthlyTrend from "./MonthlyTrend.svelte";
import MostFrequent from "./MostFrequent.svelte";
import BudgetStatus from "./BudgetStatus.svelte";
import KeywordTracker from "./KeywordTracker.svelte";

/**
 * Widget registry.
 *
 * To add a new widget:
 * 1. Create a Svelte component in src/lib/widgets/ that accepts an `expenses` prop
 * 2. Import it here and add an entry to this array
 *
 * Each widget component receives:
 *   - expenses: Array<{ id, title, amount, date, category, classification_source }>
 *   - config: optional per-instance config object (for configurable widgets)
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
  {
    id: "budget-status",
    name: "Budget Status",
    description: "Current month's budget vs. actual spending with progress bar.",
    size: "half",
    component: BudgetStatus,
  },
  {
    id: "keyword-tracker",
    name: "Keyword Tracker",
    description: "Monthly spending for expenses matching a keyword.",
    size: "half",
    component: KeywordTracker,
    configurable: true,
    multiInstance: true,
  },
];

/** Default widget instances shown on a fresh dashboard. */
export const defaultWidgetInstances = [
  { widgetId: "total-stats", instanceId: "total-stats" },
  { widgetId: "spending-by-category", instanceId: "spending-by-category" },
  { widgetId: "biggest-expense", instanceId: "biggest-expense" },
];
