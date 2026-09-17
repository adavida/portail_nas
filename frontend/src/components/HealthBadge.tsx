export function HealthBadge({ status }: { status: string }) {
  return <p data-testid="health">backend: {status}</p>;
}
