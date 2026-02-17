import { useState } from "react";
import type { FormEvent } from "react";
import { Link } from "react-router-dom";
import { requestPasswordReset } from "../api/auth";
import AuthFormPage from "../components/AuthFormPage";
import StatusMessage from "../components/StatusMessage";
import SubmitButton from "../components/SubmitButton";
import { BASE_INPUT_CLASS } from "../utils/validation";
import { useFormSubmit } from "../utils/useFormSubmit";

export default function ForgotPassword() {
    const [identifier, setIdentifier] = useState("");
    const [success, setSuccess] = useState(false);
    const { submitting, error, wrapSubmit } = useFormSubmit("Unable to request password reset.");

    const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        await wrapSubmit(async () => {
            await requestPasswordReset(identifier);
            setSuccess(true);
        });
    };

    return (
        <AuthFormPage
            title="Reset Password"
            subtitle="Enter your username or email and we will send a reset link."
            onSubmit={handleSubmit}
            footer={
                <>
                    Back to{" "}
                    <Link to="/" className="text-indigo-400 hover:text-indigo-300">
                        login
                    </Link>
                </>
            }
        >
            <div>
                <label htmlFor="reset-identifier" className="block text-sm text-gray-300 mb-1">
                    Username or email
                </label>
                <input
                    id="reset-identifier"
                    className={BASE_INPUT_CLASS}
                    value={identifier}
                    onChange={(event) => setIdentifier(event.target.value)}
                    required
                />
            </div>

            <StatusMessage type="error" message={error} />
            <StatusMessage
                type="success"
                message={success ? "If an account exists, a password reset link has been sent." : null}
            />

            <SubmitButton submitting={submitting} label="Send reset link" loadingLabel="Sending..." />
        </AuthFormPage>
    );
}
