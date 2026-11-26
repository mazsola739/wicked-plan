// A dropdown in the page's own look (docs/design.md, Filter buttons): the closed trigger is a
// hexagon like the chips, with the chosen value's colour as a small dot; the open list is a dark
// panel with a dot per row, a tick on the chosen row and a quiet dark hover. It follows the listbox
// pattern: Enter, Space or Down opens it; Up/Down/Home/End move; Enter picks; Esc or Tab closes.

import { useEffect, useId, useRef, useState } from 'react';

export type DropdownOption = { value: string; label: string; tint: string };

export function Dropdown({
  label,
  value,
  options,
  align = 'left',
  onChange,
}: {
  label: string;
  value: string;
  options: DropdownOption[];
  align?: 'left' | 'right';
  onChange: (value: string) => void;
}) {
  const id = useId();
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const root = useRef<HTMLDivElement>(null);
  const button = useRef<HTMLButtonElement>(null);
  const list = useRef<HTMLDivElement>(null);
  const chosen = options.find((o) => o.value === value) ?? options[0];

  // A click anywhere else closes it.
  useEffect(() => {
    if (!open) return;
    const away = (e: PointerEvent) => {
      if (!root.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('pointerdown', away);
    return () => document.removeEventListener('pointerdown', away);
  }, [open]);

  // Opening puts focus on the list, at the chosen row.
  useEffect(() => {
    if (open) list.current?.focus();
  }, [open]);

  const show = () => {
    setActive(
      Math.max(
        0,
        options.findIndex((o) => o.value === value),
      ),
    );
    setOpen(true);
  };
  const pick = (v: string) => {
    onChange(v);
    setOpen(false);
    button.current?.focus();
  };
  const onListKey = (e: React.KeyboardEvent) => {
    const last = options.length - 1;
    switch (e.key) {
      case 'ArrowDown':
        setActive((a) => Math.min(last, a + 1));
        break;
      case 'ArrowUp':
        setActive((a) => Math.max(0, a - 1));
        break;
      case 'Home':
        setActive(0);
        break;
      case 'End':
        setActive(last);
        break;
      case 'Enter':
      case ' ':
        pick(options[active]?.value ?? value);
        break;
      case 'Escape':
        setOpen(false);
        button.current?.focus();
        break;
      case 'Tab':
        setOpen(false);
        return;
      default:
        return;
    }
    e.preventDefault();
    e.stopPropagation();
  };

  return (
    <div ref={root} className={`dropdown${open ? ' open' : ''}`}>
      <span className="label" id={`${id}-label`}>
        {label}
      </span>
      <button
        ref={button}
        type="button"
        className={`pick ${chosen?.tint ?? ''}`}
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-labelledby={`${id}-label ${id}-value`}
        onClick={() => (open ? setOpen(false) : show())}
        onKeyDown={(e) => {
          if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
            e.preventDefault();
            show();
          }
        }}
      >
        <i className="pick-dot" aria-hidden="true" />
        <span id={`${id}-value`} className="pick-value">
          {chosen?.label}
        </span>
      </button>
      {open && (
        <div
          ref={list}
          className={`dropdown-list ${align}`}
          role="listbox"
          tabIndex={-1}
          aria-labelledby={`${id}-label`}
          aria-activedescendant={`${id}-${active}`}
          onKeyDown={onListKey}
        >
          {options.map((o, i) => (
            // biome-ignore lint/a11y/useKeyWithClickEvents: the list handles the keys (listbox pattern)
            <div
              key={o.value}
              id={`${id}-${i}`}
              role="option"
              tabIndex={-1}
              aria-selected={o.value === value}
              className={`dropdown-option ${o.tint}${i === active ? ' active' : ''}`}
              onPointerEnter={() => setActive(i)}
              onClick={() => pick(o.value)}
            >
              <i className="pick-dot" aria-hidden="true" />
              <span>{o.label}</span>
              {o.value === value && (
                <svg className="dropdown-tick" viewBox="0 0 16 16" aria-hidden="true">
                  <path d="M3 8.5 L6.5 12 L13 4.5" />
                </svg>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
