import { Construction } from 'lucide-react';

type Props = {
  title: string;
  description: string;
  phase: string;
};

/** An honest placeholder: no controls that look like they do something. */
export function PlannedView({ title, description, phase }: Props) {
  return (
    <section className="placeholder-page">
      <div className="placeholder-icon" aria-hidden="true">
        <Construction size={22} />
      </div>
      <h2>{title}</h2>
      <p>{description}</p>
      <span className="status-tag">{phase}</span>
    </section>
  );
}
