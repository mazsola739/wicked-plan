// Every game's screen, by game id. Each screen lives in its own file
// (apps/web/src/games/<PascalName>.tsx, default export), except the six first games (QuizGames.tsx, BoardGames.tsx).

import type { ComponentType } from 'react';
import { HexagonGame, KnightGame, MagicGame } from './BoardGames';
import type { ScreenProps } from './common';
import { CubesGame, EquationGame, RectanglesGame } from './QuizGames';
import RulesRace from './RulesRace';
import AnimalTileAuction from './AnimalTileAuction';
import EquationHiLo from './EquationHiLo';
import ColourCountBluff from './ColourCountBluff';
import HiddenRulesRace from './HiddenRulesRace';
import VirusGame from './VirusGame';
import EitherOr from './EitherOr';
import ThievesAndCops from './ThievesAndCops';
import SecretNumber from './SecretNumber';
import MonsterAlliance from './MonsterAlliance';
import MysteryStockMarket from './MysteryStockMarket';
import PolyominoTerritory from './PolyominoTerritory';
import TreasureIsland from './TreasureIsland';
import ColourMancala from './ColourMancala';
import JewelBlackMarket from './JewelBlackMarket';
import AuctionBingo from './AuctionBingo';
import LayTheGrass from './LayTheGrass';
import MemoryPoker from './MemoryPoker';
import WordBlockTower from './WordBlockTower';
import DuplicatePhotoBuzzer from './DuplicatePhotoBuzzer';
import SpinningPuzzleWheel from './SpinningPuzzleWheel';
import MemorySceneQuiz from './MemorySceneQuiz';
import WeightBalancePuzzle from './WeightBalancePuzzle';
import HiddenFourthPlayer from './HiddenFourthPlayer';
import TimeAuction from './TimeAuction';
import MinusOne from './MinusOne';
import MetalCastPuzzle from './MetalCastPuzzle';
import SniperHoldem from './SniperHoldem';
import DicePrediction from './DicePrediction';
import Heavy13 from './Heavy13';
import RussianRoulette from './RussianRoulette';
import WallTerritory from './WallTerritory';
import PiggyBank from './PiggyBank';
import BlindGomoku from './BlindGomoku';
import RisingWaterWell from './RisingWaterWell';
import UnknownLanguage from './UnknownLanguage';
import PiecePuzzle from './PiecePuzzle';
import FindTheDifferences from './FindTheDifferences';
import SecretCode from './SecretCode';
import FenceBuilding from './FenceBuilding';
import SymmetryPuzzle from './SymmetryPuzzle';
import BigOrSmall from './BigOrSmall';
import NineMensMorris from './NineMensMorris';
import TigerAndGoat from './TigerAndGoat';
import QuestionAndTruth from './QuestionAndTruth';

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
