import { useState } from "react";
import type { FormEvent } from "react";
import { Link, Outlet } from "react-router-dom";
import { ApiError, type User } from "../api/auth";
import HealthIndicator from "./HealthIndicator";

type LayoutProps = {
  authLoading: boolean;
  user: User | null;
  onLogin: (username: string, password: string) => Promise<void>;
  onLogout: () => Promise<void>;
};

export default function Layout({ authLoading, user, onLogin, onLogout }: LayoutProps) {
  const [menuOpen, setMenuOpen] = useState(false);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [loadingLogin, setLoadingLogin] = useState(false);
  const [loginError, setLoginError] = useState<string | null>(null);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setLoadingLogin(true);
    setLoginError(null);

    try {
      await onLogin(username, password);
      setUsername("");
      setPassword("");
      setMenuOpen(false);
    } catch (error) {
      if (error instanceof ApiError) {
        setLoginError(error.message);
      } else {
        setLoginError("Unable to log in.");
      }
    } finally {
      setLoadingLogin(false);
    }
  };

  const handleLogout = async () => {
    await onLogout();
  };

  return (
    <div className="min-h-screen flex flex-col bg-gray-900 text-gray-100">
      <header className="bg-gray-800 border-b border-gray-700">
        <nav className="max-w-6xl mx-auto flex items-center justify-between px-6 py-4">
          <Link to="/" className="text-xl font-bold text-white hover:text-indigo-400 transition-colors">
            🎲 rsdice
          </Link>
          {authLoading ? (
            <span className="text-sm text-gray-400">Checking session...</span>
          ) : user ? (
            <div className="flex items-center gap-3">
              <Link to="/profile" className="text-gray-200 hover:text-white transition-colors">
                {user.username}
              </Link>
              <button
                onClick={handleLogout}
                className="px-3 py-1.5 rounded-md bg-gray-700 hover:bg-gray-600 text-sm font-medium transition-colors"
              >
                Logout
              </button>
            </div>
          ) : (
            <div className="relative">
              <button
                onClick={() => setMenuOpen((open) => !open)}
                className="px-4 py-2 rounded-md bg-indigo-600 hover:bg-indigo-500 text-sm font-semibold transition-colors"
              >
                Login
              </button>

              {menuOpen && (
                <div className="absolute right-0 mt-2 w-80 rounded-lg border border-gray-700 bg-gray-800 p-4 shadow-xl">
                  <form className="space-y-3" onSubmit={handleSubmit}>
                    <div>
                      <label className="block text-xs text-gray-400 mb-1" htmlFor="login-username">
                        Username
                      </label>
                      <input
                        id="login-username"
                        className="w-full rounded-md border border-gray-600 bg-gray-900 px-3 py-2 text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        value={username}
                        onChange={(event) => setUsername(event.target.value)}
                        required
                      />
                    </div>
                    <div>
                      <label className="block text-xs text-gray-400 mb-1" htmlFor="login-password">
                        Password
                      </label>
                      <input
                        id="login-password"
                        type="password"
                        className="w-full rounded-md border border-gray-600 bg-gray-900 px-3 py-2 text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        value={password}
                        onChange={(event) => setPassword(event.target.value)}
                        required
                      />
                    </div>

                    {loginError && <p className="text-sm text-red-400">{loginError}</p>}

                    <button
                      type="submit"
                      disabled={loadingLogin}
                      className="w-full rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white hover:bg-indigo-500 disabled:opacity-60"
                    >
                      {loadingLogin ? "Logging in..." : "Login"}
                    </button>

                    <p className="text-sm text-gray-400 text-center">
                      <Link
                        to="/forgot-password"
                        onClick={() => setMenuOpen(false)}
                        className="text-indigo-400 hover:text-indigo-300"
                      >
                        Forgot your password?
                      </Link>
                    </p>
                  </form>

                  <p className="mt-3 text-sm text-gray-400">
                    New here?{" "}
                    <Link
                      to="/register"
                      onClick={() => setMenuOpen(false)}
                      className="text-indigo-400 hover:text-indigo-300"
                    >
                      Register a new user
                    </Link>
                  </p>
                </div>
              )}
            </div>
          )}
        </nav>
      </header>

      <main className="flex-1">
        <Outlet />
      </main>

      <footer className="border-t border-gray-700 bg-gray-800">
        <div className="max-w-6xl mx-auto px-6 py-3 flex items-center justify-end">
          <HealthIndicator />
        </div>
      </footer>
    </div>
  );
}
