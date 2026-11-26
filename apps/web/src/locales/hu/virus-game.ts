// Hungarian for The Virus Game (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const NUM = '(#\\d+)';

// "Reports: #1 accuses #3; #2 accuses #5." with 1 to 10 reports.
const REPORTS: [string, string][] = Array.from({ length: 10 }, (_, k) => {
  const en = Array.from({ length: k + 1 }, () => `${NUM} accuses ${NUM}`).join('; ');
  const hu = Array.from({ length: k + 1 }, (_, i) => `$${2 * i + 1} → $${2 * i + 2}`).join('; ');
  return [`Reports: ${en}\\.`, `Vádak: ${hu}.`];
});

// "Votes: #1→#3, #2→nobody, …": one part per agent, 6 on the page; every mix of "nobody".
const VOTES: [string, string][] = Array.from({ length: 64 }, (_, mask) => {
  const en: string[] = [];
  const hu: string[] = [];
  let g = 0;
  for (let i = 0; i < 6; i++) {
    const nobody = (mask >> i) & 1;
    en.push(nobody ? `${NUM}→nobody` : `${NUM}→${NUM}`);
    hu.push(nobody ? `$${++g}→senki` : `$${++g}→$${++g}`);
  }
  return [`Votes: ${en.join(', ')}\\.`, `Szavazatok: ${hu.join(', ')}.`];
});

const hu: Locale = {
  name: 'A vírusjáték',
  short:
    'Rejtett szerepek: a terroristák vírust terjesztenek; a rendőrnek, az újságírónak, a kutatóknak és a polgároknak karanténba kell zárniuk őket.',
  strings: {
    // Roles, role texts and missions (crates/rules/src/games/virus_game.rs).
    terrorist: 'terrorista',
    officer: 'rendőr',
    journalist: 'újságíró',
    researcher: 'kutató',
    citizen: 'polgár',
    'Terrorist: whoever you meet, or who meets you, is infected.':
      'Terrorista: akivel találkozol, vagy aki veled találkozik, megfertőződik.',
    'Officer: you learn whether the person you meet is a terrorist.': 'Rendőr: megtudod, hogy akivel találkozol, terrorista-e.',
    'Journalist: you learn whom the person you meet met, and if they got infected.':
      'Újságíró: megtudod, kivel találkozott az, akivel te találkozol, és hogy az illető megfertőződött-e.',
    'Researcher: the person you meet is vaccinated (cured and immune).':
      'Kutató: akivel találkozol, beoltod (meggyógyul, és immunis lesz).',
    'Citizen: find the terrorists and stay healthy.': 'Polgár: találd meg a terroristákat, és maradj egészséges.',
    'Stay healthy to the end.': 'Maradj egészséges a végéig.',
    'Meet the same player in two rounds.': 'Találkozz ugyanazzal a játékossal két körben.',
    'Vote for a terrorist who goes into quarantine.': 'Szavazz egy terroristára, aki karanténba kerül.',
    'Never be accused.': 'Soha ne vádoljanak meg.',
    'The officer is infected at the end.': 'A végén a rendőr fertőzött legyen.',
    'Never be quarantined.': 'Soha ne kerülj karanténba.',
    // The screen.
    Infect: 'Megfertőz',
    Check: 'Ellenőriz',
    Watch: 'Megfigyel',
    Vaccinate: 'Beolt',
    Meet: 'Találkozik',
    Contact: 'Kapcsolat',
    Report: 'Jelentés',
    Vote: 'Szavazás',
    Carrier: 'Hordozó',
    Immune: 'Immunis',
    Infected: 'Fertőzött',
    Healthy: 'Egészséges',
    Accuse: 'Vádol',
    Quarantine: 'Karanténba',
    'Roles revealed': 'Kiderültek a szerepek',
    'Round {n} of {total}': '{n}. kör / {total}',
    'Mission: {mission}': 'Küldetés: {mission}',
    'not a terrorist': 'nem terrorista',
    quarantine: 'karantén',
    infected: 'fertőzött',
    'The terrorists win.': 'A terroristák nyertek.',
    'The city wins.': 'A város nyert.',
    'Your score: {n}': 'A pontszámod: {n}',
    'See results': 'Eredmények',
    'Choose whom to meet ({action}).': 'Válaszd ki, kivel találkozol ({action}).',
    'Accuse someone in public, or stay quiet.': 'Vádolj meg valakit nyilvánosan, vagy hallgass.',
    'Vote someone into quarantine, or nobody.': 'Szavazz valakit karanténba, vagy senkit.',
    'Stay quiet': 'Hallgatok',
    Nobody: 'Senki',
    "Keys: a player's number picks them, Enter confirms.": 'Billentyűk: egy játékos száma kiválasztja őt, az Enter megerősít.',
    'You are in quarantine.': 'Karanténban vagy.',
    'New infections per round: {list}': 'Új fertőzések körönként: {list}',
    Reports: 'Vádak',
    'R{n}: {by} accuses {target}': '{n}. kör: {by} → {target}',
    // Log lines without a number.
    'Round 1: choose whom to meet.': '1. kör: válaszd ki, kivel találkozol.',
    'Nobody accuses anyone.': 'Senki sem vádol senkit.',
    'Nobody goes into quarantine.': 'Senki sem kerül karanténba.',
    'The game is over: the terrorists win.': 'Vége a játéknak: a terroristák nyertek.',
    'The game is over: the city wins.': 'Vége a játéknak: a város nyert.',
  },
  // Notes, log lines and replay moments written by the rules.
  patterns: [
    [`Round (\\d+): ${NUM} is a TERRORIST\\.`, '$1. kör: $2 TERRORISTA.'],
    [`Round (\\d+): ${NUM} is not a terrorist\\.`, '$1. kör: $2 nem terrorista.'],
    [`Round (\\d+): ${NUM} met ${NUM}, who was newly infected\\.`, '$1. kör: $2 találkozója: $3, aki újonnan megfertőződött.'],
    [`Round (\\d+): ${NUM} met ${NUM}, who was not newly infected\\.`, '$1. kör: $2 találkozója: $3, aki nem fertőződött meg újonnan.'],
    [`Round (\\d+): ${NUM} met nobody\\.`, '$1. kör: $2 senkivel sem találkozott.'],
    ['Round (\\d+): everyone has met someone\\. New infections: (\\d+)\\.', '$1. kör: mindenki találkozott valakivel. Új fertőzések: $2.'],
    ...REPORTS,
    ...VOTES,
    [`${NUM} goes into quarantine and is a TERRORIST\\.`, '$1 karanténba kerül: TERRORISTA.'],
    [`${NUM} goes into quarantine and is not a terrorist\\.`, '$1 karanténba kerül: nem terrorista.'],
    ['(\\d+) new infections in one round', '$1 új fertőzés egyetlen körben'],
    ['(\\{[a-z0-9-]+\\}) quarantined: the last terrorist caught', '$1 karanténban: az utolsó terrorista is lebukott'],
  ],
};

export default hu;
