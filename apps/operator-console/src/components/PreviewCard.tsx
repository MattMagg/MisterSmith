
import type { ResultPreview } from '../types';
import { KeyValueGrid } from './KeyValueGrid';
import { prettyJson } from '../utils/format';

export interface PreviewCardProps {
  preview?: ResultPreview | null;
}

export function PreviewCard({ preview }: PreviewCardProps) {
  if (!preview) {
    return (
      <section className="subpanel">
        <h3>Result preview</h3>
        <p>No shared result preview is available for this workflow yet.</p>
      </section>
    );
  }

  return (
    <section className="subpanel">
      <h3>Result preview</h3>
      <KeyValueGrid
        rows={[
          ['Workflow', preview.workflow_id],
          ['Proof outcome', preview.proof_outcome],
          ['Payload', preview.payload_location],
        ]}
      />
      <p className="preview-copy">{preview.preview_text ?? 'No preview text recorded.'}</p>
      <details className="payload-disclosure">
        <summary>Provenance lines</summary>
        <pre>{prettyJson(preview.provenance_lines)}</pre>
      </details>
    </section>
  );
}
