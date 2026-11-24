// Every game's screen, by game id. Each screen lives in its own file
// (apps/web/src/games/<PascalName>.tsx, default export), except the six first games (QuizGames.tsx, BoardGames.tsx).

import type { ComponentType } from 'react';
import AnimalTileAuction from './AnimalTileAuction';
import AuctionBingo from './AuctionBingo';
import BigOrSmall from './BigOrSmall';
import BlindGomoku from './BlindGomoku';
import { HexagonGame, KnightGame, MagicGame } from './BoardGames';
import ColourCountBluff from './ColourCountBluff';
import ColourMancala from './ColourMancala';
import type { ScreenProps } from './common';
import DicePrediction from './DicePrediction';
import DuplicatePhotoBuzzer from './DuplicatePhotoBuzzer';
import EitherOr from './EitherOr';
import EquationHiLo from './EquationHiLo';
import FenceBuilding from './FenceBuilding';
import FindTheDifferences from './FindTheDifferences';
import Heavy13 from './Heavy13';
import HiddenFourthPlayer from './HiddenFourthPlayer';
import HiddenRulesRace from './HiddenRulesRace';
import JewelBlackMarket from './JewelBlackMarket';
import LayTheGrass from './LayTheGrass';
import MemoryPoker from './MemoryPoker';
import MemorySceneQuiz from './MemorySceneQuiz';
import MetalCastPuzzle from './MetalCastPuzzle';
import MinusOne from './MinusOne';
import MonsterAlliance from './MonsterAlliance';
import MysteryStockMarket from './MysteryStockMarket';
import NineMensMorris from './NineMensMorris';
import PiecePuzzle from './PiecePuzzle';
import PiggyBank from './PiggyBank';
import PolyominoTerritory from './PolyominoTerritory';
import QuestionAndTruth from './QuestionAndTruth';
import { CubesGame, EquationGame, RectanglesGame } from './QuizGames';
import RisingWaterWell from './RisingWaterWell';
import RulesRace from './RulesRace';
import RussianRoulette from './RussianRoulette';
import SecretCode from './SecretCode';
import SecretNumber from './SecretNumber';
import SniperHoldem from './SniperHoldem';
import SpinningPuzzleWheel from './SpinningPuzzleWheel';
import SymmetryPuzzle from './SymmetryPuzzle';
import ThievesAndCops from './ThievesAndCops';
import TigerAndGoat from './TigerAndGoat';
import TimeAuction from './TimeAuction';
import TreasureIsland from './TreasureIsland';
import UnknownLanguage from './UnknownLanguage';
import VirusGame from './VirusGame';
import WallTerritory from './WallTerritory';
import WeightBalancePuzzle from './WeightBalancePuzzle';
import WordBlockTower from './WordBlockTower';

export const SCREENS: Record<string, ComponentType<ScreenProps>> = {
  'count-the-cubes': CubesGame,
  'count-the-rectangles': RectanglesGame,
  'equation-target': EquationGame,
  hexagon: HexagonGame,
  'knights-tour': KnightGame,
  'magic-square': MagicGame,
  'rules-race': RulesRace,
  'animal-tile-auction': AnimalTileAuction,
  'equation-hi-lo': EquationHiLo,
  'colour-count-bluff': ColourCountBluff,
  'hidden-rules-race': HiddenRulesRace,
  'virus-game': VirusGame,
  'either-or': EitherOr,
  'thieves-and-cops': ThievesAndCops,
  'secret-number': SecretNumber,
  'monster-alliance': MonsterAlliance,
  'mystery-stock-market': MysteryStockMarket,
  'polyomino-territory': PolyominoTerritory,
  'treasure-island': TreasureIsland,
  'colour-mancala': ColourMancala,
  'jewel-black-market': JewelBlackMarket,
  'auction-bingo': AuctionBingo,
  'lay-the-grass': LayTheGrass,
  'memory-poker': MemoryPoker,
  'word-block-tower': WordBlockTower,
  'duplicate-photo-buzzer': DuplicatePhotoBuzzer,
  'spinning-puzzle-wheel': SpinningPuzzleWheel,
  'memory-scene-quiz': MemorySceneQuiz,
  'weight-balance-puzzle': WeightBalancePuzzle,
  'hidden-fourth-player': HiddenFourthPlayer,
  'time-auction': TimeAuction,
  'minus-one': MinusOne,
  'metal-cast-puzzle': MetalCastPuzzle,
  'sniper-holdem': SniperHoldem,
  'dice-prediction': DicePrediction,
  'heavy-13': Heavy13,
  'russian-roulette': RussianRoulette,
  'wall-territory': WallTerritory,
  'piggy-bank': PiggyBank,
  'blind-gomoku': BlindGomoku,
  'rising-water-well': RisingWaterWell,
  'unknown-language': UnknownLanguage,
  'piece-puzzle': PiecePuzzle,
  'find-the-differences': FindTheDifferences,
  'secret-code': SecretCode,
  'fence-building': FenceBuilding,
  'symmetry-puzzle': SymmetryPuzzle,
  'big-or-small': BigOrSmall,
  'nine-mens-morris': NineMensMorris,
  'tiger-and-goat': TigerAndGoat,
  'question-and-truth': QuestionAndTruth,
};
