import { useCallback, useEffect, useState } from "react";
import { Navigate, Route, Routes } from "react-router-dom";
import Layout from "./components/Layout";
import PrivateRoute from "./components/PrivateRoute";
import { getCurrentUser, login, logout, refreshSession, type User } from "./api/auth";
import Home from "./pages/Home";
import Game from "./pages/Game";
import Games from "./pages/Games";
import Profile from "./pages/Profile";
import Register from "./pages/Register";
import VerifyEmail from "./pages/VerifyEmail";
import ForgotPassword from "./pages/ForgotPassword";
import ResetPassword from "./pages/ResetPassword";
import Rules from "./pages/Rules";

function App() {
  const [currentUser, setCurrentUser] = useState<User | null>(null);
  const [authLoading, setAuthLoading] = useState(true);

  const refreshCurrentUser = useCallback(async () => {
    try {
      const user = await getCurrentUser();
      setCurrentUser(user);
    } catch {
      setCurrentUser(null);
    }
  }, []);

  useEffect(() => {
    const bootstrapAuth = async () => {
      const user = await refreshSession();
      setCurrentUser(user);
      setAuthLoading(false);
    };

    bootstrapAuth();
  }, [refreshCurrentUser]);

  const handleLogin = async (username: string, password: string) => {
    const user = await login({ username, password });
    setCurrentUser(user);
  };

  const handleLogout = async () => {
    try {
      await logout();
    } finally {
      setCurrentUser(null);
    }
  };

  return (
    <Routes>
      <Route
        element={
          <Layout
            authLoading={authLoading}
            user={currentUser}
            onLogin={handleLogin}
            onLogout={handleLogout}
          />
        }
      >
        <Route path="/" element={<Home />} />
        <Route
          path="/games"
          element={<Games authLoading={authLoading} isAuthenticated={currentUser !== null} />}
        />
        <Route path="/game/:id" element={<Game />} />
        <Route path="/game" element={<Navigate to="/games" replace />} />
        <Route path="/rules" element={<Rules />} />
        <Route path="/register" element={<Register onRegistered={setCurrentUser} />} />
        <Route path="/forgot-password" element={<ForgotPassword />} />
        <Route path="/reset-password" element={<ResetPassword />} />
        <Route path="/verify-email" element={<VerifyEmail onVerified={refreshCurrentUser} />} />
        <Route
          path="/profile"
          element={
            <PrivateRoute authLoading={authLoading} user={currentUser}>
              {currentUser && <Profile user={currentUser} />}
            </PrivateRoute>
          }
        />
      </Route>
    </Routes>
  );
}

export default App;
