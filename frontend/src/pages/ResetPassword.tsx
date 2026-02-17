import { useState } from "react";
import type { FormEvent } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { ApiError, resetPassword } from "../api/auth";
import PasswordRequirements from "../components/PasswordRequirements";
import { checkPassword, fieldClass, isPasswordValid } from "../utils/validation";

export default function ResetPassword() {
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const token = params.get("token")?.trim() ?? "";

  const [password, setPassword] = useState("");
  const [repeatPassword, setRepeatPassword] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [touched, setTouched] = useState({ password: false, repeatPassword: false });
  const touch = (field: keyof typeof touched) => setTouched((t) => ({ ...t, [field]: true }));

  const passwordRules = checkPassword(password);
  const passwordValid = isPasswordValid(passwordRules);
  const repeatValid = password.length > 0 && repeatPassword === password;

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setTouched({ password: true, repeatPassword: true });

    if (!token) {
      setError("Missing password reset token.");
      return;
    }

    if (!passwordValid || !repeatValid) return;

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
            className={fieldClass(touched.password, passwordValid)}
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            onBlur={() => touch("password")}
            required
          />
          {password.length > 0 && <PasswordRequirements rules={passwordRules} />}
        </div>

        <div>
          <label htmlFor="repeat-password" className="block text-sm text-gray-300 mb-1">
            Repeat new password
          </label>
          <input
            id="repeat-password"
            type="password"
            className={fieldClass(touched.repeatPassword, repeatValid)}
            value={repeatPassword}
            onChange={(event) => setRepeatPassword(event.target.value)}
            onBlur={() => touch("repeatPassword")}
            required
          />
          {touched.repeatPassword && !repeatValid && repeatPassword.length > 0 && (
            <p className="mt-1 text-xs text-red-400">Passwords do not match.</p>
          )}
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
