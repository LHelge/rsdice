import { useState } from "react";
import type { FormEvent } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { ApiError, resetPassword } from "../api/auth";

export default function ResetPassword() {
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const token = params.get("token")?.trim() ?? "";

  const [password, setPassword] = useState("");
  const [repeatPassword, setRepeatPassword] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!token) {
      setError("Missing password reset token.");
      return;
    }

    if (password !== repeatPassword) {
      setError("Passwords do not match.");
      return;
    }

    setSubmitting(true);
    setError(null);

    try {
      await resetPassword(token, password);
      navigate("/");
    } catch (submitError) {
      if (submitError instanceof ApiError) {
        setError(submitError.message);
      } else {
        setError("Unable to reset password.");
      }
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="max-w-md mx-auto px-6 py-16">
      <h1 className="text-3xl font-bold text-white mb-2">Choose New Password</h1>
      <p className="text-gray-400 mb-8">Enter a new password for your account.</p>

      <form className="space-y-4 bg-gray-800 p-6 rounded-xl border border-gray-700" onSubmit={handleSubmit}>
        <div>
          <label htmlFor="new-password" className="block text-sm text-gray-300 mb-1">
            New password
          </label>
          <input
            id="new-password"
            type="password"
            className="w-full rounded-md border border-gray-600 bg-gray-900 px-3 py-2 text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            required
          />
          <p className="mt-1 text-xs text-gray-500">Use at least 10 chars with upper/lowercase, number and symbol.</p>
        </div>

        <div>
          <label htmlFor="repeat-password" className="block text-sm text-gray-300 mb-1">
            Repeat new password
          </label>
          <input
            id="repeat-password"
            type="password"
            className="w-full rounded-md border border-gray-600 bg-gray-900 px-3 py-2 text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
            value={repeatPassword}
            onChange={(event) => setRepeatPassword(event.target.value)}
            required
          />
        </div>

        {error && <p className="text-sm text-red-400">{error}</p>}

        <button
          type="submit"
          disabled={submitting}
          className="w-full rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-500 disabled:opacity-60"
        >
          {submitting ? "Resetting..." : "Reset password"}
        </button>
      </form>

      <p className="mt-6 text-sm text-gray-400">
        Back to <Link to="/" className="text-indigo-400 hover:text-indigo-300">login</Link>
      </p>
    </div>
  );
}
