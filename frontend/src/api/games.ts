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
    const response = await fetch("/api/games", {
        credentials: "same-origin",
    });

    if (!response.ok) {
        throw new Error((await response.text()) || "Failed to fetch games");
    }

    return (await response.json()) as GameListItem[];
}

export async function createGame(): Promise<GameSnapshot> {
    const response = await fetch("/api/games", {
        method: "PUT",
        credentials: "same-origin",
    });

    if (!response.ok) {
        throw new Error((await response.text()) || "Failed to create game");
    }

    return (await response.json()) as GameSnapshot;
}
