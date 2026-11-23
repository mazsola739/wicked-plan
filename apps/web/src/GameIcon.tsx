// Each game's neon icon: the user's art, one file per game in assets/icons/<game-id>.webp
// (docs/design.md, Visual identity > Game icons). Loaded only when it comes into view.

const ICONS = import.meta.glob<string>('./assets/icons/*.webp', { eager: true, import: 'default' });

export function GameIcon({ id, className = '' }: { id: string; className?: string }) {
  const src = ICONS[`./assets/icons/${id}.webp`];
  if (!src) return null;
  return <img className={`game-icon ${className}`} src={src} alt="" loading="lazy" decoding="async" />;
}
