type StatusMessageProps = {
    type: "error" | "success";
    message: string | null;
};

/** Renders a coloured status message, or nothing when `message` is null. */
export default function StatusMessage({ type, message }: StatusMessageProps) {
    if (!message) return null;
    return (
        <p className={`text-sm ${type === "error" ? "text-ctp-red" : "text-ctp-green"}`}>
            {message}
        </p>
    );
}
