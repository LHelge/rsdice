import { useState } from "react";
import { checkPassword, isPasswordValid } from "./validation";

/**
 * Manages state and validation for a new-password + repeat-password field pair.
 *
 * - `password` / `repeat` – controlled field values and their setters.
 * - `touched` / `touch(field)` – per-field blur tracking so errors only appear
 *   after the user has interacted with a field.
 * - `touchAll()` – marks both fields as touched (used on form submit).
 * - `rules` – the `PasswordRules` object from `checkPassword`.
 * - `passwordValid` / `repeatValid` – derived validity flags.
 * - `reset()` – clears all state back to the initial empty/untouched values.
 */
export function usePasswordFields() {
    const [password, setPassword] = useState("");
    const [repeat, setRepeat] = useState("");
    const [touched, setTouched] = useState({ password: false, repeat: false });

    const rules = checkPassword(password);
    const passwordValid = isPasswordValid(rules);
    const repeatValid = password.length > 0 && repeat === password;

    const touch = (field: "password" | "repeat") =>
        setTouched((t) => ({ ...t, [field]: true }));

    const touchAll = () => setTouched({ password: true, repeat: true });

    const reset = () => {
        setPassword("");
        setRepeat("");
        setTouched({ password: false, repeat: false });
    };

    return {
        password,
        setPassword,
        repeat,
        setRepeat,
        touched,
        touch,
        touchAll,
        rules,
        passwordValid,
        repeatValid,
        reset,
    };
}
