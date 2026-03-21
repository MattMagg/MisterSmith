

export interface KeyValueGridProps {
  rows: Array<[string, string]>;
}

export function KeyValueGrid({ rows }: KeyValueGridProps) {
  return (
    <dl className="key-value-grid">
      {rows.map(([label, value]) => (
        <div key={label}>
          <dt>{label}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}
