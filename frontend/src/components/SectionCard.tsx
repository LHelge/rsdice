import type { ReactNode } from "react";

type SectionCardProps = {
    /** Lucide icon (or any element) displayed next to the heading. */
    icon: ReactNode;
    title: string;
    children: ReactNode;
};

/**
 * A gray-800 card with an icon heading and arbitrary content body.
 * Used for feature highlights (Home) and rule sections (Rules).
 */
export default function SectionCard({ icon, title, children }: SectionCardProps) {
    return (
        <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
            <h2 className="flex items-center gap-2 text-xl font-semibold mb-3 text-indigo-400">
                {icon}
                {title}
            </h2>
            {children}
        </div>
    );
}
