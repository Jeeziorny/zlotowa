export const CHART_PALETTE = [
  "#f59e0b", // amber-500 (primary)
  "#d97706", // amber-600
  "#b45309", // amber-700
  "#92400e", // amber-800
  "#eab308", // yellow-500
  "#ca8a04", // yellow-600
  "#a16207", // yellow-700
  "#854d0e", // yellow-800
];

export function chartColor(index) {
  return CHART_PALETTE[index % CHART_PALETTE.length];
}

export function formatAmount(value) {
  return value.toFixed(2);
}
