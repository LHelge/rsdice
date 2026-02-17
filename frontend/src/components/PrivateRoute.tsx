import type { ReactNode } from "react";
import { Navigate } from "react-router-dom";
import type { User } from "../api/auth";

type PrivateRouteProps = {
    authLoading: boolean;
    user: User | null;
    /** The protected content to render when the user is authenticated. */
    children: ReactNode;
};

/**
 * Renders children when authenticated, a loading indicator while the session
 * is being checked, or a redirect to `/` when there is no user.
 */
export default function PrivateRoute({ authLoading, user, children }: PrivateRouteProps) {
    if (authLoading) {
        return (
            <div className="max-w-3xl mx-auto px-6 py-16 text-center text-ctp-subtext1">
                Loading profile...
            </div>
        );
    }

    if (!user) {
        return <Navigate to="/" replace />;
    }

    return <>{children}</>;
}
