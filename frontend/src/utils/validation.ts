export interface PasswordRules {
    length: boolean;
    uppercase: boolean;
    lowercase: boolean;
    number: boolean;
    symbol: boolean;
}

export function checkPassword(password: string): PasswordRules {
    return {
        length: password.length >= 10,
        uppercase: /[A-Z]/.test(password),
        lowercase: /[a-z]/.test(password),
        number: /[0-9]/.test(password),
        symbol: /[^A-Za-z0-9]/.test(password),
    };
}

export function isPasswordValid(rules: PasswordRules): boolean {
    return Object.values(rules).every(Boolean);
}

/** Base Tailwind classes for a plain (non-validated) text input. */
export const BASE_INPUT_CLASS =
    "w-full rounded-md border border-ctp-surface1 bg-ctp-base px-3 py-2 text-sm text-ctp-text focus:outline-none focus:ring-2 focus:ring-ctp-mauve";

/** Returns Tailwind border + focus-ring classes based on touched/valid state. */
export function fieldClass(touched: boolean, valid: boolean): string {
    const base =
        "w-full rounded-md border bg-ctp-base px-3 py-2 text-sm text-ctp-text focus:outline-none focus:ring-2";
    if (!touched) return `${base} border-ctp-surface1 focus:ring-ctp-mauve`;
    if (valid) return `${base} border-ctp-green focus:ring-ctp-green`;
    return `${base} border-ctp-red focus:ring-ctp-red`;
}
