// Hungarian for Unknown Language (i18n.ts). The English text is the key.
// The puzzle itself (the translated carvings and the answer options) stays in English: the script
// encodes English words and word order. Only the screen around it is translated.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Ismeretlen nyelv',
  short: 'Fejts meg egy képírást néhány lefordított véset alapján, aztán olvass és írj vele.',
  strings: {
    'What does this say?': 'Mit jelent ez?',
    'How is this written? “{text}”': 'Hogyan írják ezt? „{text}”',
    'What does this word mean?': 'Mit jelent ez a szó?',
    'Stage {stage} of 3 · question {n} of {total}': '{stage}. szakasz / 3 · {n}. kérdés / {total}',
    'Carvings with a translation': 'Vésetek fordítással',
    'Things and actions.': 'Dolgok és cselekvések.',
    'Now with qualities.': 'Most tulajdonságokkal is.',
    'Known roots with a new mark: what do they mean?': 'Ismert tövek új jellel: mit jelentenek?',
    'Right, +{n}': 'Jó, +{n}',
    'Not right. ': 'Nem jó. ',
    'It was:': 'A helyes válasz:',
    'Keys: 1–4 pick an option.': 'Billentyűk: 1–4 választ.',
  },
  // Replay moments written by the rules (crates/rules/src/games/unknown_language.rs).
  patterns: [
    ['Perfect run: all (\\d+) carvings read right', 'Hibátlan: $1/$1 véset jól olvasva'],
    ['Stage 3 cracked without a miss', 'A 3. szakasz megfejtve, hiba nélkül'],
  ],
};

export default hu;
