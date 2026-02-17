import { useEffect, useRef } from "react";
import { useParams } from "react-router-dom";

type GameWasmModule = {
    default: () => Promise<unknown>;
    set_game_id: (gameId: string) => void;
};

// Module-level singletons — survive component unmount/remount.
// wasm-bindgen's init function must only be called once per page lifetime.
let wasmModule: GameWasmModule | null = null;
let wasmInitialized = false;
let wasmInitPromise: Promise<void> | null = null;

async function ensureWasmReady(): Promise<GameWasmModule> {
    if (!wasmModule) {
        wasmModule = (await import("../wasm/game.js")) as GameWasmModule;
    }

    if (!wasmInitialized) {
        if (!wasmInitPromise) {
            wasmInitPromise = wasmModule.default().then(() => {
                wasmInitialized = true;
            });
        }
        await wasmInitPromise;
    }

    return wasmModule;
}

export default function Game() {
    const { id } = useParams<{ id: string }>();
    const containerRef = useRef<HTMLDivElement>(null);

    // Move the persistent canvas into our container, then back to <body> on
    // unmount. This keeps the canvas (and Bevy's reference to it) alive across
    // React navigation while visually placing it inside the page layout.
    useEffect(() => {
        const canvas = document.getElementById("bevy-canvas") as HTMLCanvasElement | null;
        const container = containerRef.current;
        if (!canvas || !container) return;

        canvas.style.display = "block";
        canvas.className = "w-full h-full rounded-lg shadow-lg";
        container.appendChild(canvas);

        return () => {
            canvas.style.display = "none";
            document.body.appendChild(canvas);
        };
    }, []);

    useEffect(() => {
        if (!id) return;

        ensureWasmReady()
            .then((wasm) => wasm.set_game_id(id))
            .catch((err) => console.error("Failed to load WASM game module:", err));
    }, [id]);

    return (
        <div className="flex items-center justify-center h-full">
            <div ref={containerRef} className="w-[800px] h-[600px]" />
        </div>
    );
}
