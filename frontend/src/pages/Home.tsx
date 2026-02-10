import { Link } from "react-router-dom";

export default function Home() {
  return (
    <div className="max-w-4xl mx-auto px-6 py-16">
      <section className="text-center mb-16">
        <h1 className="text-5xl font-bold mb-4">rsdice</h1>
        <p className="text-xl text-gray-400">
          A turn-based online multiplayer dice game
        </p>
        <Link
          to="/game"
          className="inline-block mt-8 px-8 py-3 bg-indigo-600 hover:bg-indigo-500 text-white font-semibold rounded-lg transition-colors"
        >
          Play Now
        </Link>
      </section>

      <section className="grid gap-8 md:grid-cols-2">
        <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
          <h2 className="text-lg font-semibold mb-3 text-indigo-400">🗺️ Conquer the Map</h2>
          <p className="text-gray-300">
            Compete on a hex-tile map divided into areas. Each player starts
            with randomly assigned areas, each holding 1–8 dice. Attack
            adjacent enemy areas to expand your territory.
          </p>
        </div>

        <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
          <h2 className="text-lg font-semibold mb-3 text-indigo-400">🎲 Roll to Attack</h2>
          <p className="text-gray-300">
            Combat is resolved by dice rolls — the attacker and defender each
            roll the dice on their area. Roll higher than your opponent to
            capture their territory, but beware: a tie goes to the defender.
          </p>
        </div>

        <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
          <h2 className="text-lg font-semibold mb-3 text-indigo-400">🔗 Connect Your Areas</h2>
          <p className="text-gray-300">
            At the end of your turn you receive bonus dice equal to your
            largest group of connected areas. Build and protect contiguous
            territory to grow stronger each round.
          </p>
        </div>

        <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
          <h2 className="text-lg font-semibold mb-3 text-indigo-400">🏆 Last One Standing</h2>
          <p className="text-gray-300">
            Players are eliminated when they lose all their areas. Outplay
            your opponents, manage your reserves (up to 60 stored dice), and
            be the last player standing to win.
          </p>
        </div>
      </section>
    </div>
  );
}
