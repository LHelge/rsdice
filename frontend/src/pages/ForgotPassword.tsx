import { useState } from "react";
import type { FormEvent } from "react";
import { Link } from "react-router-dom";
import { ApiError, requestPasswordReset } from "../api/auth";

export default function ForgotPassword() {
  const [identifier, setIdentifier] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSubmitting(true);
    setError(null);

    try {
      await requestPasswordReset(identifier);
      setSuccess(true);
    } catch (submitError) {
      if (submitError instanceof ApiError) {
        setError(submitError.message);
      } else {
        setError("Unable to request password reset.");
      }
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="max-w-md mx-auto px-6 py-16">
      <h1 className="text-3xl font-bold text-white mb-2">Reset Password</h1>
      <p className="text-gray-400 mb-8">Enter your username or email and we will send a reset link.</p>

      <form className="space-y-4 bg-gray-800 p-6 rounded-xl border border-gray-700" onSubmit={handleSubmit}>
        <div>
          <label htmlFor="reset-identifier" className="block text-sm text-gray-300 mb-1">
            Username or email
          </label>
          <input
            id="reset-identifier"
            className="w-full rounded-md border border-gray-600 bg-gray-900 px-3 py-2 text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
            value={identifier}
            onChange={(event) => setIdentifier(event.target.value)}
            required
          />
        </div>

        {error && <p className="text-sm text-red-400">{error}</p>}
        {success && (
          <p className="text-sm text-green-400">
            If an account exists, a password reset link has been sent.
          </p>
        )}

        <button
          type="submit"
          disabled={submitting}
          className="w-full rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-500 disabled:opacity-60"
        >
          {submitting ? "Sending..." : "Send reset link"}
        </button>
      </form>

      <p className="mt-6 text-sm text-gray-400">
        Back to <Link to="/" className="text-indigo-400 hover:text-indigo-300">login</Link>
      </p>
    </div>
  );
}
