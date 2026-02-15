export type User = {
    id: string;
    username: string;
    email: string;
    email_verified: boolean;
    admin: boolean;
};

type SessionResponse = User & {
    access_token: string;
};

type LoginRequest = {
    username: string;
    password: string;
};

type RegisterRequest = {
    username: string;
    email: string;
    password: string;
};

export class ApiError extends Error {
    status: number;

    constructor(status: number, message: string) {
        super(message);
        this.status = status;
    }
}

let accessToken: string | null = null;

function setAccessToken(token: string | null) {
    accessToken = token;
}

function parseAuthResponse(session: SessionResponse): User {
    setAccessToken(session.access_token);
    return {
        id: session.id,
        username: session.username,
        email: session.email,
        email_verified: session.email_verified,
        admin: session.admin,
    };
}

type RequestOptions = {
    auth?: boolean;
    retryOnUnauthorized?: boolean;
};

async function request<T>(path: string, init?: RequestInit, options?: RequestOptions): Promise<T> {
    const headers = new Headers(init?.headers ?? {});
    if (options?.auth && accessToken) {
        headers.set("Authorization", `Bearer ${accessToken}`);
    }

    const response = await fetch(path, {
        credentials: "same-origin",
        headers,
        ...init,
    });

    if (response.status === 401 && options?.auth && options.retryOnUnauthorized !== false) {
        const refreshed = await refreshSession();
        if (refreshed) {
            return request(path, init, { ...options, retryOnUnauthorized: false });
        }
    }

    if (!response.ok) {
        const message = (await response.text()) || "Request failed";
        throw new ApiError(response.status, message);
    }

    if (response.status === 204) {
        return undefined as T;
    }

    const contentType = response.headers.get("content-type") ?? "";
    if (!contentType.includes("application/json")) {
        return undefined as T;
    }

    return (await response.json()) as T;
}

function jsonRequest(body: unknown): RequestInit {
    return {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(body),
    };
}

export function getCurrentUser() {
    return request<User>("/api/users/me", undefined, { auth: true });
}

export async function login(payload: LoginRequest) {
    const session = await request<SessionResponse>("/api/users/auth", jsonRequest(payload));
    return parseAuthResponse(session);
}

export async function register(payload: RegisterRequest) {
    const session = await request<SessionResponse>("/api/users/register", jsonRequest(payload));
    return parseAuthResponse(session);
}

export async function refreshSession() {
    try {
        const session = await request<SessionResponse>("/api/users/refresh", { method: "POST" });
        return parseAuthResponse(session);
    } catch {
        setAccessToken(null);
        return null;
    }
}

export async function logout() {
    try {
        await request<void>("/api/users/logout", { method: "POST" });
    } finally {
        setAccessToken(null);
    }
}

export function changePassword(userId: string, currentPassword: string, password: string) {
    return request<void>(`/api/users/${userId}/password`, jsonRequest({
        current_password: currentPassword,
        password,
    }), { auth: true });
}

export function verifyEmail(token: string) {
    return request<void>("/api/users/verify-email", jsonRequest({ token }));
}

export function requestPasswordReset(identifier: string) {
    return request<void>("/api/users/request-password-reset", jsonRequest({ identifier }));
}

export function resetPassword(token: string, password: string) {
    return request<void>("/api/users/reset-password", jsonRequest({ token, password }));
}