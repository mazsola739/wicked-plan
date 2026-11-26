// The replay of a big moment (docs/design.md, Broadcast feel > Replays): the game is frozen
// behind a broadcast-style REPLAY band for a few seconds; a tap or any key skips it.

import { useEffect } from 'react';
import { nameOf } from './games/common';
import { t } from './i18n';
import { endReplay } from './play';
import type { Moment } from './types';
import './Replay.css';

/** `{seat-id}` in a moment's text stands for that seat's name. */
function fill(text: string): string {
  return text.replace(/\{([a-z0-9-]+)\}/g, (_, id: string) => nameOf(id));
}

export function Replay({ moment }: { moment: Moment }) {
  // Keys skip the replay and do not reach the game underneath (its keys listen on window too).
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey || e.altKey) return;
      e.preventDefault();
      e.stopImmediatePropagation();
      endReplay();
    };
    window.addEventListener('keydown', onKey, true);
    return () => window.removeEventListener('keydown', onKey, true);
  }, []);

  return (
    <div className="replay" role="status" aria-live="polite" onPointerDown={endReplay}>
      <div className="replay-band" key={moment.seq}>
        <span className="replay-tag">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M11 6 L3 12 L11 18 Z M21 6 L13 12 L21 18 Z" />
          </svg>
          {t('Replay')}
        </span>
        <strong className="replay-who">{nameOf(moment.by)}</strong>
        <p className="replay-text">{fill(t(moment.text))}</p>
        <span className="replay-time" aria-hidden="true" />
      </div>
      <p className="replay-skip">{t('Tap or press any key to go on')}</p>
    </div>
  );
}
