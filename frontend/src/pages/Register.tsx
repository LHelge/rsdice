import { useState } from "react";
import type { FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import { register, type User } from "../api/auth";
import AuthFormPage from "../components/AuthFormPage";
import FormField from "../components/FormField";
import PasswordRequirements from "../components/PasswordRequirements";
import StatusMessage from "../components/StatusMessage";
import SubmitButton from "../components/SubmitButton";
import { fieldClass } from "../utils/validation";
import { useFormSubmit } from "../utils/useFormSubmit";
import { usePasswordFields } from "../utils/usePasswordFields";

type RegisterProps = {
    onRegistered: (user: User) => void;
};

export default function Register({ onRegistered }: RegisterProps) {
    const navigate = useNavigate();
    const [username, setUsername] = useState("");
    const [email, setEmail] = useState("");
    const [touched, setTouched] = useState({ username: false, email: false });
    const touch = (field: "username" | "email") =>
        setTouched((t) => ({ ...t, [field]: true }));

    const {
        password, setPassword,
        repeat, setRepeat,
        touched: pwTouched, touch: touchPw, touchAll,
        rules, passwordValid, repeatValid,
    } = usePasswordFields();

    const { submitting, error, wrapSubmit } = useFormSubmit("Unable to register user.");

    const emailValid = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
    const usernameValid = username.trim().length >= 3;

    const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        setTouched({ username: true, email: true });
        touchAll();

        if (!usernameValid || !emailValid || !passwordValid || !repeatValid) return;

        await wrapSubmit(async () => {
            const user = await register({ username, email, password });
            onRegistered(user);
            navigate("/profile");
        });
    };

    return (
        <AuthFormPage
            title="Create Account"
            subtitle="Sign up to start playing rsdice."
            onSubmit={handleSubmit}
            footer={
                <>
                    Already have an account?{" "}
                    <Link to="/" className="text-indigo-400 hover:text-indigo-300">
                        Go to login
                    </Link>
                </>
            }
        >
            <FormField
                label="Username"
                id="register-username"
                touched={touched.username}
                valid={usernameValid}
                error="Username must be at least 3 characters."
            >
                <input
                    id="register-username"
                    className={fieldClass(touched.username, usernameValid)}
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    onBlur={() => touch("username")}
                    required
                />
            </FormField>

            <FormField
                label="Email"
                id="register-email"
                touched={touched.email}
                valid={emailValid}
                error="Enter a valid email address."
            >
                <input
                    id="register-email"
                    type="email"
                    className={fieldClass(touched.email, emailValid)}
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    onBlur={() => touch("email")}
                    required
                />
            </FormField>

            <FormField
                label="Password"
                id="register-password"
                touched={pwTouched.password}
                valid={passwordValid}
            >
                <input
                    id="register-password"
                    type="password"
                    className={fieldClass(pwTouched.password, passwordValid)}
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    onBlur={() => touchPw("password")}
                    required
                />
                {password.length > 0 && <PasswordRequirements rules={rules} />}
            </FormField>

            <FormField
                label="Repeat password"
                id="register-repeat-password"
                touched={pwTouched.repeat}
                valid={repeatValid}
                error={repeat.length > 0 ? "Passwords do not match." : undefined}
            >
                <input
                    id="register-repeat-password"
                    type="password"
                    className={fieldClass(pwTouched.repeat, repeatValid)}
                    value={repeat}
                    onChange={(e) => setRepeat(e.target.value)}
                    onBlur={() => touchPw("repeat")}
                    required
                />
            </FormField>

            <StatusMessage type="error" message={error} />
            <SubmitButton submitting={submitting} label="Register" loadingLabel="Creating account..." />
        </AuthFormPage>
    );
}