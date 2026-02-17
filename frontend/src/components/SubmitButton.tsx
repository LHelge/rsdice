type SubmitButtonProps = {
    /** Whether the form is currently being submitted. Disables the button. */
    submitting: boolean;
    /** Button label in the idle state. */
    label: string;
    /** Button label while submitting. */
    loadingLabel: string;
    /** When `true` (default) the button spans the full container width. */
    fullWidth?: boolean;
};

/** Indigo submit button with a loading label and disabled state. */
export default function SubmitButton({
    submitting,
    label,
    loadingLabel,
    fullWidth = true,
}: SubmitButtonProps) {
    return (
        <button
            type="submit"
            disabled={submitting}
            className={`${fullWidth ? "w-full " : ""}rounded-md bg-ctp-mauve px-4 py-2 text-sm font-semibold text-ctp-base hover:bg-ctp-lavender disabled:opacity-60`}
        >
            {submitting ? loadingLabel : label}
        </button>
    );
}
