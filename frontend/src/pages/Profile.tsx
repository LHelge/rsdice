import { useState } from "react";
import type { FormEvent } from "react";
import { ApiError, changePassword, type User } from "../api/auth";

type ProfileProps = {
    user: User;
};

export default function Profile({ user }: ProfileProps) {
    const [password, setPassword] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [success, setSuccess] = useState<string | null>(null);

    const handlePasswordChange = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        setSubmitting(true);
        setError(null);
        setSuccess(null);

        try {
            await changePassword(user.id, password);
            setPassword("");
            setSuccess("Password updated successfully.");
        } catch (submitError) {
            if (submitError instanceof ApiError) {
                setError(submitError.message);
            } else {
                setError("Unable to update password.");
            }
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <div className="max-w-3xl mx-auto px-6 py-16 space-y-8">
            <section className="bg-gray-800 border border-gray-700 rounded-xl p-6">
                <h1 className="text-3xl font-bold mb-4">Profile</h1>
                <dl className="grid gap-3 sm:grid-cols-2 text-sm">
                    <div>
                        <dt className="text-gray-400">Username</dt>
                        <dd className="text-gray-100">{user.username}</dd>
                    </div>
                    <div>
                        <dt className="text-gray-400">Email</dt>
                        <dd className="text-gray-100">{user.email}</dd>
                    </div>
                    <div>
                        <dt className="text-gray-400">Email verification</dt>
                        <dd className="text-gray-100">{user.email_verified ? "Verified" : "Not verified"}</dd>
                    </div>
                    <div>
                        <dt className="text-gray-400">Role</dt>
                        <dd className="text-gray-100">{user.admin ? "Admin" : "Player"}</dd>
                    </div>
                </dl>
            </section>

            <section className="bg-gray-800 border border-gray-700 rounded-xl p-6">
                <h2 className="text-xl font-semibold mb-4">Change Password</h2>
                <form className="space-y-4" onSubmit={handlePasswordChange}>
                    <div>
                        <label htmlFor="profile-password" className="block text-sm text-gray-300 mb-1">
                            New password
                        </label>
                        <input
                            id="profile-password"
                            type="password"
                            className="w-full rounded-md border border-gray-600 bg-gray-900 px-3 py-2 text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                            value={password}
                            onChange={(event) => setPassword(event.target.value)}
                            required
                        />
                        <p className="mt-1 text-xs text-gray-500">Use at least 10 chars with upper/lowercase, number and symbol.</p>
                    </div>

                    {error && <p className="text-sm text-red-400">{error}</p>}
                    {success && <p className="text-sm text-green-400">{success}</p>}

                    <button
                        type="submit"
                        disabled={submitting}
                        className="rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-500 disabled:opacity-60"
                    >
                        {submitting ? "Updating..." : "Update password"}
                    </button>
                </form>
            </section>
        </div>
    );
}