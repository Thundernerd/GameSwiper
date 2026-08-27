/** Shared API contract — frontend types; backend JSON must match these shapes. */

export type SwipeAction = 'like' | 'dislike' | 'defer';

export interface User {
  id: number;
  username: string;
}

export interface AuthCredentials {
  username: string;
  password: string;
}

export interface AuthResponse {
  user: User;
}

export interface EventSummary {
  id: number;
  name: string;
  slug: string;
  logoUrl: string | null;
  startTime: string | null;
  endTime: string | null;
  ratedCount: number;
  totalCount: number;
}

export interface EventsListResponse {
  events: EventSummary[];
}

export interface MediaItem {
  kind: 'video' | 'image';
  url: string;
  title?: string | null;
}

export interface GameCard {
  id: number;
  name: string;
  genres: string[];
  platforms: string[];
  media: MediaItem[];
  coverUrl: string | null;
  summary: string | null;
  rating: number | null;
  developers: string[];
  publishers: string[];
  /** ISO date string when known; optional for older fixtures. */
  releaseDate?: string | null;
  /** External links; optional on queue cards, required on library. */
  links?: GameLink[];
}

export interface EventRef {
  id: number;
  name: string;
  slug: string;
}

export interface QueueResponse {
  event: EventRef;
  games: GameCard[];
}

export interface SwipeRequest {
  gameId: number;
  action: SwipeAction;
}

export interface SwipeResponse {
  ok: true;
  remaining: number;
}

export interface GameLink {
  label: string;
  url: string;
}

export interface GameOverview {
  id: number;
  name: string;
  coverUrl: string | null;
  platforms: string[];
  links: GameLink[];
}

export interface OverviewResponse {
  event: EventRef;
  liked: GameOverview[];
  disliked: GameOverview[];
}

export interface LibraryGame {
  id: number;
  name: string;
  genres: string[];
  platforms: string[];
  media: MediaItem[];
  coverUrl: string | null;
  summary: string | null;
  rating: number | null;
  developers: string[];
  publishers: string[];
  links: GameLink[];
  releaseDate: string | null;
  events: EventRef[];
}

export interface LibraryResponse {
  liked: LibraryGame[];
  disliked: LibraryGame[];
}
