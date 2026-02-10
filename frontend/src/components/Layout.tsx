import { Link, Outlet } from "react-router-dom";

export default function Layout() {
  return (
    <div className="min-h-screen flex flex-col bg-gray-900 text-gray-100">
      <header className="bg-gray-800 border-b border-gray-700">
        <nav className="max-w-6xl mx-auto flex items-center justify-between px-6 py-4">
          <Link to="/" className="text-xl font-bold text-white hover:text-indigo-400 transition-colors">
            🎲 rsdice
          </Link>
          <div className="flex gap-6">
            <Link to="/" className="text-gray-300 hover:text-white transition-colors">
              Home
            </Link>
            <Link to="/game" className="text-gray-300 hover:text-white transition-colors">
              Play
            </Link>
          </div>
        </nav>
      </header>

      <main className="flex-1">
        <Outlet />
      </main>
    </div>
  );
}
