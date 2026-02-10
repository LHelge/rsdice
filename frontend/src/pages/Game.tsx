import { useEffect, useRef } from "react";

export default function Game() {
    const initializedRef = useRef(false);

    useEffect(() => {
        if (initializedRef.current) return;
        initializedRef.current = true;

        async function loadWasm() {
            try {
                const wasm = await import("../wasm/game.js");
                await wasm.default();
            } catch (err) {
                console.error("Failed to load WASM game module:", err);
            }
        }

        loadWasm();
    }, []);

    return (
        <div className="flex items-center justify-center h-full">
            <div className="w-[800px] h-[600px]">
                <canvas id="bevy-canvas" className="rounded-lg shadow-lg" />
            </div>
        </div>
    );
}
