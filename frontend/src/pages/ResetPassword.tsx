import type { FormEvent } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { resetPassword } from "../api/auth";
import AuthFormPage from "../components/AuthFormPage";
import FormField from "../components/FormField";
import PasswordRequirements from "../components/PasswordRequirements";
import StatusMessage from "../components/StatusMessage";
import SubmitButton from "../components/SubmitButton";
import { fieldClass } from "../utils/validation";
import { useFormSubmit } from "../utils/useFormSubmit";
import { usePasswordFields } from "../utils/usePasswordFields";

export default function ResetPassword() {
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const token = params.get("token")?.trim() ?? "";

  const {
    password, setPassword,
    repeat, setRepeat,
    touched, touch, touchAll,
    rules, passwordValid, repeatValid,
  } = usePasswordFields();

  const { submitting, error, setError, wrapSubmit } = useFormSubmit("Unable to reset password.");

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    touchAll();

    if (!token) {
      setError("Missing password reset token.");
      return;
    }

    if (!passwordValid || !repeatValid) return;

    await wrapSubmit(async () => {
      await resetPassword(token, password);
      navigate("/");
    });
  };

  return (
    <AuthFormPage
      title="Choose New Password"
      subtitle="Enter a new password for your account."
      onSubmit={handleSubmit}
      footer={
        <>
          Back to{" "}
          <Link to="/" className="text-ctp-mauve hover:text-ctp-lavender">
            login
          </Link>
        </>
      }
    >
      <FormField
        label="New password"
        id="new-password"
        touched={touched.password}
        valid={passwordValid}
      >
        <input
          id="new-password"
          type="password"
          className={fieldClass(touched.password, passwordValid)}
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          onBlur={() => touch("password")}
          required
        />
        {password.length > 0 && <PasswordRequirements rules={rules} />}
      </FormField>

      <FormField
        label="Repeat new password"
        id="repeat-password"
        touched={touched.repeat}
        valid={repeatValid}
        error={repeat.length > 0 ? "Passwords do not match." : undefined}
      >
        <input
          id="repeat-password"
          type="password"
          className={fieldClass(touched.repeat, repeatValid)}
          value={repeat}
          onChange={(e) => setRepeat(e.target.value)}
          onBlur={() => touch("repeat")}
          required
        />
      </FormField>

      <StatusMessage type="error" message={error} />
      <SubmitButton submitting={submitting} label="Reset password" loadingLabel="Resetting..." />
    </AuthFormPage>
  );
}
