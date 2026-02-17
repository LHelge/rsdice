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

/** Returns Tailwind border + focus-ring classes based on touched/valid state. */
export function fieldClass(touched: boolean, valid: boolean): string {
    const base =
        "w-full rounded-md border bg-gray-900 px-3 py-2 text-sm text-gray-100 focus:outline-none focus:ring-2";
    if (!touched) return `${base} border-gray-600 focus:ring-indigo-500`;
    if (valid) return `${base} border-green-500 focus:ring-green-500`;
    return `${base} border-red-500 focus:ring-red-500`;
}
