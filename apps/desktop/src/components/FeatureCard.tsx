export function FeatureCard(props: { title: string; description: string }) {
  return (
    <article className="feature-card">
      <h3>{props.title}</h3>
      <p>{props.description}</p>
    </article>
  );
}
