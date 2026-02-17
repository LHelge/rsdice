import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
    createGame,
    gameStateLabel,
    isActiveGameState,
    listGames,
    type GameListItem,
} from "../api/games";

type GamesProps = {
    authLoading: boolean;
    isAuthenticated: boolean;
};

function sortByNewest(a: GameListItem, b: GameListItem): number {
    return b.id.localeCompare(a.id);
}

export default function Games({ authLoading, isAuthenticated }: GamesProps) {
    const navigate = useNavigate();

    const [games, setGames] = useState<GameListItem[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [creating, setCreating] = useState(false);

    useEffect(() => {
        let mounted = true;

        const bootstrap = async () => {
            try {
                const initialGames = await listGames();
                if (!mounted) return;
                setGames(initialGames);
                setError(null);
            } catch {
                if (!mounted) return;
                setError("Unable to load games.");
            } finally {
                if (mounted) {
                    setLoading(false);
                }
            }
        };

        bootstrap();

        const source = new EventSource("/api/games/stream", { withCredentials: true });

        const onGamesEvent = (event: MessageEvent<string>) => {
            try {
                const payload = JSON.parse(event.data) as GameListItem[];
                setGames(payload);
                setError(null);
            } catch {
                setError("Received invalid game stream payload.");
            }
        };

        source.addEventListener("games", onGamesEvent as EventListener);

        source.onerror = () => {
            setError((current) => current ?? "Live updates temporarily unavailable.");
        };

        return () => {
            mounted = false;
            source.removeEventListener("games", onGamesEvent as EventListener);
            source.close();
        };
    }, []);

    const activeGames = useMemo(
        () => games.filter((game) => isActiveGameState(game.state)).sort(sortByNewest),
        [games],
    );

    const handleCreateGame = async () => {
        setCreating(true);
        setError(null);

        try {
            const game = await createGame();
            navigate(`/game/${game.id}`);
        } catch {
            setError("Unable to create game.");
        } finally {
            setCreating(false);
        }
    };

    return (
        <div className="max-w-5xl mx-auto px-6 py-12 space-y-8">
            <section className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
                <div>
                    <h1 className="text-3xl font-bold text-ctp-text">Active Games</h1>
                    <p className="text-ctp-subtext0 mt-1">Live list of games in waiting or in-progress state.</p>
                </div>

                {!authLoading && (
                    <button
                        type="button"
                        onClick={handleCreateGame}
                        disabled={creating || !isAuthenticated}
                        title={!isAuthenticated ? "Log in to create a game" : undefined}
                        className="px-4 py-2 rounded-md bg-ctp-mauve hover:bg-ctp-lavender disabled:opacity-50 disabled:cursor-not-allowed font-semibold text-ctp-base"
                    >
                        {creating ? "Creating..." : "Create Game"}
                    </button>
                )}
            </section>

            {error && (
                <div className="rounded-md border border-ctp-red/40 bg-ctp-red/10 px-4 py-3 text-ctp-red">
                    {error}
                </div>
            )}

            {loading ? (
                <div className="rounded-lg border border-ctp-surface1 bg-ctp-surface0 px-6 py-10 text-center text-ctp-subtext1">
                    Loading games...
                </div>
            ) : activeGames.length === 0 ? (
                <div className="rounded-lg border border-ctp-surface1 bg-ctp-surface0 px-6 py-10 text-center text-ctp-subtext1">
                    No active games right now.
                </div>
            ) : (
                <ul className="space-y-3">
                    {activeGames.map((game) => (
                        <li
                            key={game.id}
                            className="rounded-lg border border-ctp-surface1 bg-ctp-surface0 px-4 py-4 flex flex-col gap-3 md:flex-row md:items-center md:justify-between"
                        >
                            <div className="space-y-1">
                                <p className="text-sm text-ctp-subtext0">Game {game.id}</p>
                                <p className="text-ctp-subtext1">
                                    Creator: <span className="font-medium text-ctp-text">{game.creator.name}</span>
                                </p>
                                <p className="text-sm text-ctp-subtext1">
                                    Players: {game.player_count} · State: {gameStateLabel(game.state)}
                                </p>
                            </div>

                            {!authLoading && (
                                <button
                                    type="button"
                                    onClick={() => isAuthenticated && navigate(`/game/${game.id}`)}
                                    disabled={!isAuthenticated}
                                    title={!isAuthenticated ? "Log in to join a game" : undefined}
                                    className="self-start md:self-auto px-4 py-2 rounded-md bg-ctp-surface1 hover:bg-ctp-surface2 disabled:opacity-50 disabled:cursor-not-allowed text-sm font-semibold"
                                >
                                    Join Game
                                </button>
                            )}
                        </li>
                    ))}
                </ul>
            )}
        </div>
    );
}
