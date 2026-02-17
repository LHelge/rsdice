import { useState } from "react";
import type { FormEvent } from "react";
import { ApiError, changePassword, type User } from "../api/auth";
import PasswordRequirements from "../components/PasswordRequirements";
import { checkPassword, fieldClass, isPasswordValid } from "../utils/validation";

type ProfileProps = {
    user: User;
};

export default function Profile({ user }: ProfileProps) {
    const [currentPassword, setCurrentPassword] = useState("");
    const [newPassword, setNewPassword] = useState("");
    const [repeatNewPassword, setRepeatNewPassword] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [success, setSuccess] = useState<string | null>(null);

    const [touched, setTouched] = useState({
        currentPassword: false,
        newPassword: false,
        repeatNewPassword: false,
    });
    const touch = (field: keyof typeof touched) => setTouched((t) => ({ ...t, [field]: true }));

    const passwordRules = checkPassword(newPassword);
    const newPasswordValid = isPasswordValid(passwordRules);
    const repeatValid = newPassword.length > 0 && repeatNewPassword === newPassword;
    const currentPasswordValid = currentPassword.length > 0;

    const handlePasswordChange = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        setTouched({ currentPassword: true, newPassword: true, repeatNewPassword: true });

        if (!currentPasswordValid || !newPasswordValid || !repeatValid) return;

        setSubmitting(true);
        setError(null);
        setSuccess(null);

        try {
            await changePassword(user.id, currentPassword, newPassword);
            setCurrentPassword("");
            setNewPassword("");
            setRepeatNewPassword("");
            setTouched({ currentPassword: false, newPassword: false, repeatNewPassword: false });
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
                        <label htmlFor="profile-current-password" className="block text-sm text-gray-300 mb-1">
                            Current password
                        </label>
                        <input
                            id="profile-current-password"
                            type="password"
                            className={fieldClass(touched.currentPassword, currentPasswordValid)}
                            value={currentPassword}
                            onChange={(event) => setCurrentPassword(event.target.value)}
                            onBlur={() => touch("currentPassword")}
                            required
                        />
                        {touched.currentPassword && !currentPasswordValid && (
                            <p className="mt-1 text-xs text-red-400">Enter your current password.</p>
                        )}
                    </div>

                    <div>
                        <label htmlFor="profile-password" className="block text-sm text-gray-300 mb-1">
                            New password
                        </label>
                        <input
                            id="profile-password"
                            type="password"
                            className={fieldClass(touched.newPassword, newPasswordValid)}
                            value={newPassword}
                            onChange={(event) => setNewPassword(event.target.value)}
                            onBlur={() => touch("newPassword")}
                            required
                        />
                        {newPassword.length > 0 && <PasswordRequirements rules={passwordRules} />}
                    </div>

                    <div>
                        <label htmlFor="profile-repeat-password" className="block text-sm text-gray-300 mb-1">
                            Repeat new password
                        </label>
                        <input
                            id="profile-repeat-password"
                            type="password"
                            className={fieldClass(touched.repeatNewPassword, repeatValid)}
                            value={repeatNewPassword}
                            onChange={(event) => setRepeatNewPassword(event.target.value)}
                            onBlur={() => touch("repeatNewPassword")}
                            required
                        />
                        {touched.repeatNewPassword && !repeatValid && repeatNewPassword.length > 0 && (
                            <p className="mt-1 text-xs text-red-400">Passwords do not match.</p>
                        )}
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