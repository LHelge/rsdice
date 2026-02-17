import type { FormEvent, ReactNode } from "react";

type AuthFormPageProps = {
    title: string;
    subtitle: string;
    /** Content rendered inside the `<form>` element. */
    children: ReactNode;
    /** Text / elements placed below the form card (e.g. a "Back to login" link). */
    footer: ReactNode;
    onSubmit: (e: FormEvent<HTMLFormElement>) => void;
};

/**
 * Shared page wrapper for simple authentication forms.
 *
 * Renders the `max-w-md` centred column, heading, subtitle, the gray-800 form
 * card, and a small footer slot — matching the identical layout used by the
 * Register, ForgotPassword, and ResetPassword pages.
 */
export default function AuthFormPage({
    title,
    subtitle,
    children,
    footer,
    onSubmit,
}: AuthFormPageProps) {
    return (
        <div className="max-w-md mx-auto px-6 py-16">
            <h1 className="text-3xl font-bold text-white mb-2">{title}</h1>
            <p className="text-gray-400 mb-8">{subtitle}</p>

            <form
                className="space-y-4 bg-gray-800 p-6 rounded-xl border border-gray-700"
                onSubmit={onSubmit}
            >
                {children}
            </form>

            <p className="mt-6 text-sm text-gray-400">{footer}</p>
        </div>
    );
}
