

export interface PrioritySelectProps {
  value: string;
  onChange: (value: string) => void;
}

export function PrioritySelect(props: PrioritySelectProps) {
  return (
    <label className="field field-inline">
      <span>Priority</span>
      <select
        value={props.value}
        onChange={(event) => props.onChange(event.target.value)}
      >
        <option value="urgent">urgent</option>
        <option value="high">high</option>
        <option value="normal">normal</option>
        <option value="low">low</option>
        <option value="background">background</option>
      </select>
    </label>
  );
}
