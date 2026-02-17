import { useState } from "react";
import type { FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import { ApiError, register, type User } from "../api/auth";
import PasswordRequirements from "../components/PasswordRequirements";
import { checkPassword, fieldClass, isPasswordValid } from "../utils/validation";

type RegisterProps = {
    onRegistered: (user: User) => void;
};

export default function Register({ onRegistered }: RegisterProps) {
    const navigate = useNavigate();
    const [username, setUsername] = useState("");
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [repeatPassword, setRepeatPassword] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const [touched, setTouched] = useState({
        username: false,
        email: false,
        password: false,
        repeatPassword: false,
    });

    const touch = (field: keyof typeof touched) =>
        setTouched((t) => ({ ...t, [field]: true }));

    const passwordRules = checkPassword(password);
    const passwordValid = isPasswordValid(passwordRules);
    const emailValid = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
    const usernameValid = username.trim().length >= 3;
    const repeatValid = password.length > 0 && repeatPassword === password;

    const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        setTouched({ username: true, email: true, password: true, repeatPassword: true });

        if (!usernameValid || !emailValid || !passwordValid || !repeatValid) return;

        setSubmitting(true);
        setError(null);

        try {
            const user = await register({ username, email, password });
            onRegistered(user);
            navigate("/profile");
        } catch (submitError) {
            if (submitError instanceof ApiError) {
                setError(submitError.message);
            } else {
                setError("Unable to register user.");
            }
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <div className="max-w-md mx-auto px-6 py-16">
            <h1 className="text-3xl font-bold text-white mb-2">Create Account</h1>
            <p className="text-gray-400 mb-8">Sign up to start playing rsdice.</p>

            <form className="space-y-4 bg-gray-800 p-6 rounded-xl border border-gray-700" onSubmit={handleSubmit}>
                <div>
                    <label htmlFor="register-username" className="block text-sm text-gray-300 mb-1">
                        Username
                    </label>
                    <input
                        id="register-username"
                        className={fieldClass(touched.username, usernameValid)}
                        value={username}
                        onChange={(event) => setUsername(event.target.value)}
                        onBlur={() => touch("username")}
                        required
                    />
                    {touched.username && !usernameValid && (
                        <p className="mt-1 text-xs text-red-400">Username must be at least 3 characters.</p>
                    )}
                </div>

                <div>
                    <label htmlFor="register-email" className="block text-sm text-gray-300 mb-1">
                        Email
                    </label>
                    <input
                        id="register-email"
                        type="email"
                        className={fieldClass(touched.email, emailValid)}
                        value={email}
                        onChange={(event) => setEmail(event.target.value)}
                        onBlur={() => touch("email")}
                        required
                    />
                    {touched.email && !emailValid && (
                        <p className="mt-1 text-xs text-red-400">Enter a valid email address.</p>
                    )}
                </div>

                <div>
                    <label htmlFor="register-password" className="block text-sm text-gray-300 mb-1">
                        Password
                    </label>
                    <input
                        id="register-password"
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
                    <label htmlFor="register-repeat-password" className="block text-sm text-gray-300 mb-1">
                        Repeat password
                    </label>
                    <input
                        id="register-repeat-password"
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
                    {submitting ? "Creating account..." : "Register"}
                </button>
            </form>

            <p className="mt-6 text-sm text-gray-400">
                Already have an account? <Link to="/" className="text-indigo-400 hover:text-indigo-300">Go to login</Link>
            </p>
        </div>
    );
}