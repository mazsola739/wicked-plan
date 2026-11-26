// Each game's icon: the owner's art, one file per game. The metal set (assets/icons/<game-id>.webp)
// is used everywhere; the neon set (assets/icons-neon/<game-id>.webp) shows on the How to play
// monitor (docs/design.md, Visual identity > Game icons). Loaded only when it comes into view.

const ICONS = import.meta.glob<string>('./assets/icons/*.webp', { eager: true, import: 'default' });
const NEON = import.meta.glob<string>('./assets/icons-neon/*.webp', { eager: true, import: 'default' });

export function GameIcon({ id, className = '', neon = false }: { id: string; className?: string; neon?: boolean }) {
  const src = neon ? NEON[`./assets/icons-neon/${id}.webp`] : ICONS[`./assets/icons/${id}.webp`];
  if (!src) return null;
  return <img className={`game-icon ${className}`} src={src} alt="" loading="lazy" decoding="async" />;
}
