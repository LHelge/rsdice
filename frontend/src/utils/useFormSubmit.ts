import { useState } from "react";
import { ApiError } from "../api/auth";

/**
 * Manages `submitting` and `error` state for form submissions.
 *
 * Call `wrapSubmit(action)` inside your form handler (after calling
 * `e.preventDefault()` and any synchronous validation). The helper sets
 * `submitting = true`, clears the previous error, awaits the action, and
 * catches `ApiError`/generic errors automatically.
 */
export function useFormSubmit(fallbackError: string) {
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const wrapSubmit = async (action: () => Promise<void>) => {
        setSubmitting(true);
        setError(null);
        try {
            await action();
        } catch (err) {
            setError(err instanceof ApiError ? err.message : fallbackError);
        } finally {
            setSubmitting(false);
        }
    };

    return { submitting, error, setError, wrapSubmit };
}
