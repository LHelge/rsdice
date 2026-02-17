type StatusMessageProps = {
    type: "error" | "success";
    message: string | null;
};

/** Renders a coloured status message, or nothing when `message` is null. */
export default function StatusMessage({ type, message }: StatusMessageProps) {
    if (!message) return null;
    return (
        <p className={`text-sm ${type === "error" ? "text-red-400" : "text-green-400"}`}>
            {message}
        </p>
    );
}
