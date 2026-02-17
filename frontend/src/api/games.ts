import { request } from "./auth";

export type GameState = "WaitingForPlayers" | { InProgress: { turn: number } } | "Finished";

export type GameCreator = {
    id: string;
    name: string;
};

export type GameListItem = {
    id: string;
    creator: GameCreator;
    player_count: number;
    state: GameState;
};

export type GameSnapshot = {
    id: string;
};

export function isActiveGameState(state: GameState): boolean {
    if (state === "WaitingForPlayers") {
        return true;
    }

    if (typeof state === "object" && state !== null) {
        return "InProgress" in state;
    }

    return false;
}

export function gameStateLabel(state: GameState): string {
    if (state === "WaitingForPlayers") {
        return "Waiting";
    }

    if (state === "Finished") {
        return "Finished";
    }

    if (typeof state === "object" && state !== null && "InProgress" in state) {
        return "In Progress";
    }

    return "Unknown";
}

export async function listGames(): Promise<GameListItem[]> {
    return request<GameListItem[]>("/api/games");
}

export async function createGame(): Promise<GameSnapshot> {
    return request<GameSnapshot>("/api/games", { method: "PUT" }, { auth: true });
}
