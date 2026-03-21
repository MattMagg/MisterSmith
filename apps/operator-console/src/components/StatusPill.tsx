

export interface StatusPillProps {
  label: string;
  value: string;
  tone: 'good' | 'warn' | 'bad' | 'neutral';
  testId?: string;
}

export function StatusPill({ label, value, tone, testId }: StatusPillProps) {
  return (
    <div className={`status-pill ${tone}`} data-testid={testId}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
