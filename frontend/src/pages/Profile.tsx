import { useState } from "react";
import type { FormEvent } from "react";
import { changePassword, type User } from "../api/auth";
import FormField from "../components/FormField";
import PasswordRequirements from "../components/PasswordRequirements";
import StatusMessage from "../components/StatusMessage";
import SubmitButton from "../components/SubmitButton";
import { fieldClass } from "../utils/validation";
import { useFormSubmit } from "../utils/useFormSubmit";
import { usePasswordFields } from "../utils/usePasswordFields";

type ProfileProps = {
    user: User;
};

export default function Profile({ user }: ProfileProps) {
    const [currentPassword, setCurrentPassword] = useState("");
    const [currentTouched, setCurrentTouched] = useState(false);
    const currentPasswordValid = currentPassword.length > 0;

    const {
        password: newPassword, setPassword: setNewPassword,
        repeat: repeatNewPassword, setRepeat: setRepeatNewPassword,
        touched, touch, touchAll,
        rules, passwordValid: newPasswordValid, repeatValid,
        reset: resetPasswordFields,
    } = usePasswordFields();

    const { submitting, error, wrapSubmit } = useFormSubmit("Unable to update password.");
    const [success, setSuccess] = useState<string | null>(null);

    const handlePasswordChange = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        setCurrentTouched(true);
        touchAll();

        if (!currentPasswordValid || !newPasswordValid || !repeatValid) return;

        setSuccess(null);
        await wrapSubmit(async () => {
            await changePassword(user.id, currentPassword, newPassword);
            setCurrentPassword("");
            setCurrentTouched(false);
            resetPasswordFields();
            setSuccess("Password updated successfully.");
        });
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
                    <FormField
                        label="Current password"
                        id="profile-current-password"
                        touched={currentTouched}
                        valid={currentPasswordValid}
                        error="Enter your current password."
                    >
                        <input
                            id="profile-current-password"
                            type="password"
                            className={fieldClass(currentTouched, currentPasswordValid)}
                            value={currentPassword}
                            onChange={(e) => setCurrentPassword(e.target.value)}
                            onBlur={() => setCurrentTouched(true)}
                            required
                        />
                    </FormField>

                    <FormField
                        label="New password"
                        id="profile-password"
                        touched={touched.password}
                        valid={newPasswordValid}
                    >
                        <input
                            id="profile-password"
                            type="password"
                            className={fieldClass(touched.password, newPasswordValid)}
                            value={newPassword}
                            onChange={(e) => setNewPassword(e.target.value)}
                            onBlur={() => touch("password")}
                            required
                        />
                        {newPassword.length > 0 && <PasswordRequirements rules={rules} />}
                    </FormField>

                    <FormField
                        label="Repeat new password"
                        id="profile-repeat-password"
                        touched={touched.repeat}
                        valid={repeatValid}
                        error={repeatNewPassword.length > 0 ? "Passwords do not match." : undefined}
                    >
                        <input
                            id="profile-repeat-password"
                            type="password"
                            className={fieldClass(touched.repeat, repeatValid)}
                            value={repeatNewPassword}
                            onChange={(e) => setRepeatNewPassword(e.target.value)}
                            onBlur={() => touch("repeat")}
                            required
                        />
                    </FormField>

                    <StatusMessage type="error" message={error} />
                    <StatusMessage type="success" message={success} />

                    <SubmitButton
                        submitting={submitting}
                        label="Update password"
                        loadingLabel="Updating..."
                        fullWidth={false}
                    />
                </form>
            </section>
        </div>
    );
}