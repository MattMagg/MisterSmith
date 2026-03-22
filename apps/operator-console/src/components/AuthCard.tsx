

export interface AuthCardProps {
  title: string;
  summary: string;
  tone: 'good' | 'warn' | 'bad' | 'neutral';
  meta: string[];
  actionLabel?: string;
  onAction?: () => void;
  disabled?: boolean;
}

export function AuthCard(props: AuthCardProps) {
  const { title, summary, tone, meta, actionLabel, onAction, disabled } = props;
  const metaValues = Array.from(
    new Set(meta.map((value) => value.trim()).filter((value) => value.length > 0)),
  );

  return (
    <section className={`auth-card ${tone}`}>
      <div className="auth-header">
        <p className="eyebrow">{title}</p>
        <p className="auth-summary">{summary}</p>
      </div>
      <div className="auth-meta">
        {metaValues.map((value) => (
          <span key={value}>{value}</span>
        ))}
      </div>
      {actionLabel && onAction ? (
        <button
          className="secondary-button auth-action"
          type="button"
          onClick={onAction}
          disabled={disabled}
        >
          {actionLabel}
        </button>
      ) : null}
    </section>
  );
}
