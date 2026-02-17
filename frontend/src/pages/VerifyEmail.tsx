import { useEffect, useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { ApiError, verifyEmail } from "../api/auth";

type VerifyEmailProps = {
    onVerified: () => Promise<void>;
};

type ViewState = "verifying" | "success" | "error";

export default function VerifyEmail({ onVerified }: VerifyEmailProps) {
    const [params] = useSearchParams();
    const token = params.get("token")?.trim();
    const [state, setState] = useState<ViewState>("verifying");
    const [message, setMessage] = useState("Verifying your email...");

    useEffect(() => {
        if (!token) {
            return;
        }

        const runVerification = async () => {
            try {
                await verifyEmail(token);
                await onVerified();
                setState("success");
                setMessage("Your email has been verified successfully.");
            } catch (error) {
                setState("error");
                if (error instanceof ApiError) {
                    setMessage(error.message);
                } else {
                    setMessage("Unable to verify email.");
                }
            }
        };

        runVerification();
    }, [onVerified, token]);

    if (!token) {
        return (
            <div className="max-w-xl mx-auto px-6 py-20">
                <div className="bg-ctp-surface0 border border-ctp-surface1 rounded-xl p-8 text-center">
                    <h1 className="text-3xl font-bold mb-4">Email Verification</h1>
                    <p className="text-ctp-red">Missing verification token.</p>
                    <div className="mt-8">
                        <Link to="/" className="inline-block rounded-md bg-ctp-mauve px-4 py-2 text-sm font-semibold text-ctp-base hover:bg-ctp-lavender">
                            Back to home
                        </Link>
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className="max-w-xl mx-auto px-6 py-20">
            <div className="bg-ctp-surface0 border border-ctp-surface1 rounded-xl p-8 text-center">
                <h1 className="text-3xl font-bold mb-4">Email Verification</h1>

                {state === "verifying" && <p className="text-ctp-subtext1">{message}</p>}
                {state === "success" && <p className="text-ctp-green">{message}</p>}
                {state === "error" && <p className="text-ctp-red">{message}</p>}

                <div className="mt-8">
                    <Link to="/" className="inline-block rounded-md bg-ctp-mauve px-4 py-2 text-sm font-semibold text-ctp-base hover:bg-ctp-lavender">
                        Back to home
                    </Link>
                </div>
            </div>
        </div>
    );
}