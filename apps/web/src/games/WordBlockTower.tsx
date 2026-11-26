// Word Tower: the team uses every letter block, one topic word per player.
// Rules page: docs/games/word-block-tower.md

import { useState } from 'react';
import './WordBlockTower.css';
import { t } from '../i18n';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';

interface Block {
  letter: string;
  usedBy: number | null;
}

interface RoundResult {
  topic: string;
  words: (string | null)[];
  takeBacks: number;
  points: number;
}

interface TowerView {
  players: string[];
  you: number | null;
  round: number;
  rounds: number;
  topic: string;
  blocks: Block[];
  words: (string | null)[];
  order: number[];
  turn: number | null;
  yourTurn: boolean;
  lastWord: boolean;
  canTakeBack: boolean;
  takeBacks: number;
  results: RoundResult[];
  score: number;
  over: boolean;
}

function Tiles({ word, small }: { word: string; small?: boolean }) {
  return (
    <span className={`wbt-tiles${small ? ' small' : ''}`}>
      {word.split('').map((c, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: letters of a word are positional
        <span key={i} className="wbt-tile">
          {c}
        </span>
      ))}
    </span>
  );
}

export default function WordBlockTower({ view: raw }: ScreenProps) {
  const view = raw as TowerView;
  const you = view.you ?? 0;
  const stamp = `${view.round}:${view.words.map((w) => w ?? '-').join(',')}`;
  const [picked, setPicked] = useState<number[]>([]);
  const [forStamp, setForStamp] = useState(stamp);
  if (forStamp !== stamp) {
    setForStamp(stamp);
    setPicked([]);
  }
  const free = view.blocks.map((b, i) => ({ ...b, i })).filter((b) => b.usedBy === null);
  const word = picked.map((i) => view.blocks[i]?.letter ?? '').join('');
  const pick = (i: number) => view.yourTurn && !picked.includes(i) && setPicked((p) => [...p, i]);
  const build = () => view.yourTurn && word.length > 0 && move({ type: 'word', word });
  useKeys((key) => {
    if (!view.yourTurn) return false;
    if (/^[a-zA-Z]$/.test(key)) {
      const b = free.find((x) => x.letter === key.toUpperCase() && !picked.includes(x.i));
      if (b) pick(b.i);
    } else if (key === 'Backspace') setPicked((p) => p.slice(0, -1));
    else if (key === 'Enter') build();
    else return false;
    return true;
  });
  const who = (seat: number) => (seat === you ? t('You') : nameOf(view.players[seat]!));
  return (
    <Play
      top={<GameTop title="Word Tower" step={t('Round {n} of {total}', { n: view.round, total: view.rounds })} score={view.score} />}
      stage={
        <div className="card wbt-stage">
          <div className="wbt-topic">
            <span className="label">{t('Topic')}</span>
            <strong>{view.topic}</strong>
          </div>
          <ol className="list wbt-tower">
            {view.order.map((seat) => {
              const w = view.words[seat];
              return (
                <li key={seat} className={`${seat === you ? 'me' : ''}${seat === view.turn ? ' wbt-now' : ''}`}>
                  <span>{who(seat)}</span>
                  {w ? <Tiles word={w} small /> : <span className="muted small">{seat === view.turn ? t('building…') : t('waiting')}</span>}
                </li>
              );
            })}
          </ol>
          <div className="wbt-pile">
            {free.map((b) => (
              <button
                key={b.i}
                type="button"
                className={`wbt-block${picked.includes(b.i) ? ' picked' : ''}`}
                disabled={!view.yourTurn || picked.includes(b.i)}
                onClick={() => pick(b.i)}
              >
                {b.letter}
              </button>
            ))}
            {free.length === 0 && <span className="muted small">{t('No blocks left.')}</span>}
          </div>
        </div>
      }
      controls={
        <>
          <p className="ask">
            {view.yourTurn
              ? view.lastWord
                ? t('Your word: use every block left ({n}).', { n: free.length })
                : t('Your word: one {topic} word from the blocks.', { topic: view.topic.toLowerCase() })
              : t('{name} is building…', { name: who(view.turn ?? 0) })}
          </p>
          <div className="wbt-word">{word ? <Tiles word={word} /> : <span className="muted">{t('tap blocks')}</span>}</div>
          <div className="row">
            <button type="button" className="btn" disabled={picked.length === 0} onClick={() => setPicked((p) => p.slice(0, -1))}>
              ⌫
            </button>
            <button type="button" className="btn" disabled={picked.length === 0} onClick={() => setPicked([])}>
              {t('Clear')}
            </button>
            <button type="button" className="btn primary" disabled={!view.yourTurn || word.length === 0} onClick={build}>
              {t('Build')}
            </button>
          </div>
          {view.yourTurn && view.canTakeBack && (
            <button type="button" className="btn ghost" onClick={() => move({ type: 'takeBack' })}>
              {t('Take back the word before (−30)')}
            </button>
          )}
          {view.results.length > 0 && (
            <ul className="list wbt-results">
              {view.results.map((r) => (
                <li key={r.topic}>
                  <span>
                    {r.topic}: <span className="muted small">{r.words.filter(Boolean).join(', ')}</span>
                  </span>
                  <span className="score">+{r.points}</span>
                </li>
              ))}
            </ul>
          )}
          <KeyHint>{t('Keys: type letters, Backspace undoes, Enter builds.')}</KeyHint>
        </>
      }
    />
  );
}
