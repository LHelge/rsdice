import { useEffect, useRef } from "react";
import { useParams } from "react-router-dom";

type GameWasmModule = {
    default: () => Promise<unknown>;
    set_game_id: (gameId: string) => void;
};

export default function Game() {
    const { id } = useParams<{ id: string }>();
    const initializedRef = useRef(false);
    const wasmRef = useRef<GameWasmModule | null>(null);

    useEffect(() => {
        async function loadWasm() {
            try {
                if (!id) {
                    return;
                }

                if (!wasmRef.current) {
                    wasmRef.current = (await import("../wasm/game.js")) as GameWasmModule;
                }

                if (!initializedRef.current) {
                    initializedRef.current = true;
                    await wasmRef.current.default();
                }

                wasmRef.current.set_game_id(id);
            } catch (err) {
                console.error("Failed to load WASM game module:", err);
            }
        }

        loadWasm();
    }, [id]);

    return (
        <div className="flex items-center justify-center h-full">
            <div className="w-[800px] h-[600px]">
                <canvas id="bevy-canvas" className="rounded-lg shadow-lg" />
            </div>
        </div>
    );
}
