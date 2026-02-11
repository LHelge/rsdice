import { useEffect, useState } from "react";

type Status = "unknown" | "healthy" | "unhealthy";

export default function HealthIndicator() {
  const [status, setStatus] = useState<Status>("unknown");

  useEffect(() => {
    const check = async () => {
      try {
        const res = await fetch("/api/health");
        if (res.ok && (await res.text()) === "OK") {
          setStatus("healthy");
        } else {
          setStatus("unhealthy");
        }
      } catch {
        setStatus("unhealthy");
      }
    };

    check();
    const interval = setInterval(check, 10_000);
    return () => clearInterval(interval);
  }, []);

  const color: Record<Status, string> = {
    unknown: "bg-gray-500",
    healthy: "bg-green-500",
    unhealthy: "bg-red-500",
  };

  return (
    <div className="flex items-center gap-1.5" title={`API: ${status}`}>
      <span className={`inline-block h-2.5 w-2.5 rounded-full ${color[status]}`} />
      <span className="text-xs text-gray-400">API</span>
    </div>
  );
}
