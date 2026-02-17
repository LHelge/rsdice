import type { ReactNode } from "react";

type FormFieldProps = {
    /** The `<label>` text. */
    label: string;
    /** The `id` linking `<label htmlFor>` to the input. */
    id: string;
    /** Whether the user has already interacted with this field. */
    touched: boolean;
    /** Whether the current value passes validation. */
    valid: boolean;
    /** The error text shown below the input when touched and invalid. */
    error?: string;
    /** Optional override for the label Tailwind classes (e.g. smaller text in compact dropdowns). */
    labelClass?: string;
    /** The `<input>` element to render. */
    children: ReactNode;
};

/**
 * Wraps a form field with a label, its input, and a conditional inline error
 * message that appears only after the user has touched the field.
 */
export default function FormField({
    label,
    id,
    touched,
    valid,
    error,
    labelClass = "block text-sm text-ctp-subtext1 mb-1",
    children,
}: FormFieldProps) {
    return (
        <div>
            <label htmlFor={id} className={labelClass}>
                {label}
            </label>
            {children}
            {touched && !valid && error && (
                <p className="mt-1 text-xs text-ctp-red">{error}</p>
            )}
        </div>
    );
}
