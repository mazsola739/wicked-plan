// Mystery Stock Market: trade seven stocks with secret price rules. Rules: docs/games/mystery-stock-market.md

import { useState } from 'react';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, TimerBar, useKeys } from './common';
import './MysteryStockMarket.css';

interface StockView {
  name: string;
  ticker: string;
  price: number;
  history: number[];
  net: number[];
  holders: number[];
  rule: string | null;
}

interface MarketView {
  phase: 'trade' | 'reveal' | 'over';
  round: number;
  rounds: number;
  timeLeftMs: number;
  phaseTimeMs: number;
  stocks: StockView[];
  cash: number;
  shares: number[];
  worth: number;
  done: boolean;
  traders: { id: string; done: boolean; worth: number | null }[];
  clue: string;
  ruleTexts: [string, string][];
  log: string[];
  score: number;
}

const money = (n: number) => n.toLocaleString('en-US');

function Spark({ values }: { values: number[] }) {
  const w = 100;
  const h = 30;
  const lo = Math.min(...values);
  const hi = Math.max(...values);
  const span = hi - lo || 1;
  const pts = values
    .map((v, i) => `${values.length > 1 ? (i * w) / (values.length - 1) : 0},${h - 2 - ((v - lo) / span) * (h - 4)}`)
    .join(' ');
  const up = (values[values.length - 1] ?? 0) >= (values[0] ?? 0);
  return (
    <svg className="msm-spark" viewBox={`0 0 ${w} ${h}`} preserveAspectRatio="none" role="img" aria-label="Price chart">
      <polyline points={pts} className={up ? 'up' : 'down'} />
    </svg>
  );
}

export default function MysteryStockMarket({ view }: ScreenProps) {
  const v = view as MarketView;
  const [sel, setSel] = useState(0);
  const [qty, setQty] = useState(1);
  const trading = v.phase === 'trade' && !v.done;
  const stock = v.stocks[sel];
  const price = stock?.price ?? 1;
  const owned = v.shares[sel] ?? 0;
  const maxBuy = Math.floor(v.cash / price);
  const lastNet = stock?.net.at(-1);
  const lastHolders = stock?.holders.at(-1);
  const trade = (q: number) => trading && q !== 0 && move({ type: 'trade', stock: sel, qty: q });
  useKeys((key) => {
    if (/^[1-7]$/.test(key)) setSel(Number(key) - 1);
    else if ((key === 'b' || key === 'B') && maxBuy >= 1) trade(1);
    else if ((key === 's' || key === 'S') && owned >= 1) trade(-1);
    else if (key === 'Enter') v.phase === 'trade' ? trading && move({ type: 'done' }) : move({ type: 'close' });
    else return false;
    return true;
  });
  const end = v.phase !== 'trade';
  const ranked = [...v.traders].sort((a, b) => (b.worth ?? 0) - (a.worth ?? 0));
  return (
    <Play
      top={
        <>
          <GameTop title="Mystery Stock Market" step={end ? 'Market closed' : `Round ${v.round} of ${v.rounds}`} score={v.score} />
          <TimerBar leftMs={v.timeLeftMs} totalMs={v.phaseTimeMs} />
        </>
      }
      stage={
        <div className="msm-board">
          {v.stocks.map((s, k) => {
            const prev = s.history[s.history.length - 2];
            const change = prev ? ((s.price - prev) * 100) / prev : 0;
            return (
              <button key={s.ticker} type="button" className={`msm-row${sel === k ? ' sel' : ''}`} onClick={() => setSel(k)}>
                <span className="msm-ticker">
                  <strong>{s.ticker}</strong>
                  <small>{k + 1}</small>
                </span>
                <Spark values={s.history} />
                <span className="msm-price">
                  {money(s.price)}
                  <small className={change >= 0 ? 'up' : 'down'}>
                    {change >= 0 ? '+' : ''}
                    {change.toFixed(1)}%
                  </small>
                </span>
                <span className="msm-own">{(v.shares[k] ?? 0) > 0 ? `×${v.shares[k]}` : ''}</span>
                {s.rule && <span className="msm-rule">{s.rule}</span>}
              </button>
            );
          })}
        </div>
      }
      controls={
        <>
          <div className="msm-wallet">
            <span>
              Cash <strong>{money(v.cash)}</strong>
            </span>
            <span>
              Worth <strong>{money(v.worth)}</strong>
            </span>
          </div>
          {stock && !end && (
            <section className="msm-trade">
              <div className="msm-trade-head">
                <strong>{stock.name}</strong>
                <span>
                  {money(price)} · you hold {owned}
                </span>
              </div>
              <p className="small muted">
                Last round: {lastNet === undefined ? '–' : `${lastNet >= 0 ? '+' : ''}${lastNet} shares net`}
                {lastHolders === undefined ? '' : `, held by ${lastHolders} of ${v.traders.length}`}
              </p>
              <div className="row msm-qty">
                {[1, 5, 10].map((q) => (
                  <button key={q} type="button" className={`btn small${qty === q ? ' primary' : ''}`} onClick={() => setQty(q)}>
                    {q}
                  </button>
                ))}
              </div>
              <div className="row">
                <button type="button" className="btn" disabled={!trading || owned < 1} onClick={() => trade(-Math.min(qty, owned))}>
                  Sell {Math.min(qty, Math.max(owned, 1))}
                </button>
                <button type="button" className="btn" disabled={!trading || maxBuy < 1} onClick={() => trade(Math.min(qty, maxBuy))}>
                  Buy {Math.min(qty, Math.max(maxBuy, 1))}
                </button>
              </div>
              <div className="row">
                <button type="button" className="btn ghost small" disabled={!trading || owned < 1} onClick={() => trade(-owned)}>
                  Sell all
                </button>
                <button type="button" className="btn ghost small" disabled={!trading || maxBuy < 1} onClick={() => trade(maxBuy)}>
                  Buy max ({maxBuy})
                </button>
              </div>
            </section>
          )}
          {v.phase === 'trade' ? (
            <button type="button" className="btn primary" disabled={!trading} onClick={() => move({ type: 'done' })}>
              {trading ? 'Done for this round' : 'Waiting for the others…'}
            </button>
          ) : (
            <>
              <ul className="list">
                {ranked.map((t) => (
                  <li key={t.id} className={t.id === 'you' ? 'me' : ''}>
                    <span>{nameOf(t.id)}</span>
                    <strong>{money(t.worth ?? 0)}</strong>
                  </li>
                ))}
              </ul>
              <button type="button" className="btn primary" onClick={() => move({ type: 'close' })}>
                See results
              </button>
            </>
          )}
          <KeyHint>Keys: 1–7 pick a stock, B buys one, S sells one, Enter is Done.</KeyHint>
          <p className="small msm-clue">Your clue: {v.clue}.</p>
          <details className="msm-rules small">
            <summary>The ten possible rules</summary>
            <ul>
              {v.ruleTexts.map(([name, text]) => (
                <li key={name}>
                  <strong>{name}</strong>: {text}
                </li>
              ))}
            </ul>
          </details>
          <div className="msm-log">
            {v.log
              .slice(-4)
              .reverse()
              .map((line, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: the log only grows at the end
                <p key={i} className="small">
                  {line}
                </p>
              ))}
          </div>
        </>
      }
    />
  );
}
