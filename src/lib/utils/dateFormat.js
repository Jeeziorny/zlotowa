export const MONTH_NAMES = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];

/** "Jan 2026" — full year */
export function formatMonthLabel(ym) {
  const [y, m] = ym.split("-");
  return `${MONTH_NAMES[parseInt(m) - 1]} ${y}`;
}

/** "Jan 26" — two-digit year */
export function formatMonthFull(ym) {
  const [y, m] = ym.split("-");
  return `${MONTH_NAMES[parseInt(m) - 1]} ${y.slice(2)}`;
}

/** "Jan '26" — two-digit year with apostrophe */
export function formatMonthShort(ym) {
  const [y, m] = ym.split("-");
  return `${MONTH_NAMES[parseInt(m) - 1]} '${y.slice(2)}`;
}

/** Smart label: shows year suffix only on first item or year boundary */
export function formatMonthSmart(ym, i, data) {
  const [y, m] = ym.split("-");
  const name = MONTH_NAMES[parseInt(m) - 1];
  if (i === 0) return `${name} ${y.slice(2)}`;
  const prevYm = data[i - 1]?.ym;
  if (prevYm && prevYm.slice(0, 4) !== y) return `${name} ${y.slice(2)}`;
  return name;
}
