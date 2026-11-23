// A still, scaled-down picture of a game's opening screen for its picker card. It is the
// real screen with the game's opening view, drawn once the card comes near the viewport.

import { useEffect, useRef, useState } from 'react';
import { PreviewContext } from './games/common';
import { SCREENS } from './games/registry';
import { previewView } from './play';

// The width the screen is laid out at before scaling: what a desktop or a phone really shows.
const width = () => (window.matchMedia('(min-width: 720px)').matches ? 960 : 400);

export function Snapshot({ id }: { id: string }) {
  const box = useRef<HTMLDivElement>(null);
  const [view, setView] = useState<unknown>(null);
  const [size, setSize] = useState({ inner: 0, scale: 0 });
  useEffect(() => {
    const el = box.current;
    if (!el) return;
    const io = new IntersectionObserver(
      ([e]) => {
        if (!e?.isIntersecting) return;
        io.disconnect();
        try {
          setView(previewView(id));
        } catch {
          // No picture is better than a broken card.
        }
      },
      { rootMargin: '300px' },
    );
    io.observe(el);
    const ro = new ResizeObserver(() => {
      const inner = Math.max(width(), el.clientWidth * 2);
      setSize({ inner, scale: el.clientWidth / inner });
    });
    ro.observe(el);
    return () => {
      io.disconnect();
      ro.disconnect();
    };
  }, [id]);
  const Screen = SCREENS[id];
  return (
    <div ref={box} className="snap" aria-hidden="true">
      {view != null && Screen && size.scale > 0 && (
        <div className="snap-inner" inert style={{ width: size.inner, transform: `scale(${size.scale})` }}>
          <PreviewContext.Provider value={true}>
            <Screen view={view} />
          </PreviewContext.Provider>
        </div>
      )}
    </div>
  );
}
