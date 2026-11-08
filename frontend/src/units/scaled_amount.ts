export function scaledAmount(
  text: string,
  units: Record<string, number>,
): number {
  const unit = text.split(" ")[1] ?? "";
  return Number.parseFloat(text) * (units[unit] ?? 1);
}
