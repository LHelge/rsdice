import { useState } from "react";
import type { FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import { ApiError, register, type User } from "../api/auth";

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

    const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        if (password !== repeatPassword) {
            setError("Passwords do not match.");
            return;
        }

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
                        className="w-full rounded-md border border-gray-600 bg-gray-900 px-3 py-2 text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        value={username}
                        onChange={(event) => setUsername(event.target.value)}
                        required
                    />
                </div>

                <div>
                    <label htmlFor="register-email" className="block text-sm text-gray-300 mb-1">
                        Email
                    </label>
                    <input
                        id="register-email"
                        type="email"
                        className="w-full rounded-md border border-gray-600 bg-gray-900 px-3 py-2 text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        value={email}
                        onChange={(event) => setEmail(event.target.value)}
                        required
                    />
                </div>

                <div>
                    <label htmlFor="register-password" className="block text-sm text-gray-300 mb-1">
                        Password
                    </label>
                    <input
                        id="register-password"
                        type="password"
                        className="w-full rounded-md border border-gray-600 bg-gray-900 px-3 py-2 text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        value={password}
                        onChange={(event) => setPassword(event.target.value)}
                        required
                    />
                    <p className="mt-1 text-xs text-gray-500">Use at least 10 chars with upper/lowercase, number and symbol.</p>
                </div>

                <div>
                    <label htmlFor="register-repeat-password" className="block text-sm text-gray-300 mb-1">
                        Repeat password
                    </label>
                    <input
                        id="register-repeat-password"
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
                    {submitting ? "Creating account..." : "Register"}
                </button>
            </form>

            <p className="mt-6 text-sm text-gray-400">
                Already have an account? <Link to="/" className="text-indigo-400 hover:text-indigo-300">Go to login</Link>
            </p>
        </div>
    );
}