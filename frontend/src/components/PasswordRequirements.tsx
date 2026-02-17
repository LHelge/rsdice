import { Check, X } from "lucide-react";
import type { PasswordRules } from "../utils/validation";

const RULES: { key: keyof PasswordRules; label: string }[] = [
    { key: "length", label: "At least 10 characters" },
    { key: "uppercase", label: "Uppercase letter (A–Z)" },
    { key: "lowercase", label: "Lowercase letter (a–z)" },
    { key: "number", label: "Number (0–9)" },
    { key: "symbol", label: "Symbol (e.g. !@#$%)" },
];

type PasswordRequirementsProps = {
    rules: PasswordRules;
};

export default function PasswordRequirements({ rules }: PasswordRequirementsProps) {
    return (
        <ul className="mt-2 space-y-1">
            {RULES.map(({ key, label }) => {
                const met = rules[key];
                return (
                    <li key={key} className={`flex items-center gap-1.5 text-xs ${met ? "text-green-400" : "text-red-400"}`}>
                        {met ? <Check className="w-3.5 h-3.5 shrink-0" /> : <X className="w-3.5 h-3.5 shrink-0" />}
                        {label}
                    </li>
                );
            })}
        </ul>
    );
}
