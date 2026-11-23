// The Observer: Wicked Plan's emblem (docs/design.md, Visual identity). The emblem is the user's
// own artwork (assets/observer.png); the small crown mark is drawn here in SVG.

import observer from './assets/observer.png';
import './Emblem.css';

/** The hooded Observer with its crown and ring. */
export function Emblem({ className = '' }: { className?: string }) {
  return <img className={`emblem ${className}`} src={observer} alt="The Observer, a hooded figure with a glowing face" />;
}

/** The crown alone: the small brand mark. */
export function Crown({ className = '' }: { className?: string }) {
  return (
    <svg className={`crown ${className}`} viewBox="10 -4 80 40" aria-hidden="true">
      <path
        d="M14,32 L18,15 L30,25 L36,7 L44,21 L50,-1 L56,21 L64,7 L70,25 L82,15 L86,32 Q50,26 14,32 Z"
        fill="none"
        stroke="currentColor"
        strokeWidth={3}
        strokeLinejoin="miter"
      />
    </svg>
  );
}
