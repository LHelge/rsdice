export type User = {
    id: string;
    username: string;
    email: string;
    email_verified: boolean;
    admin: boolean;
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

async function request<T>(path: string, init?: RequestInit): Promise<T> {
    const response = await fetch(path, {
        credentials: "same-origin",
        ...init,
    });

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
    return request<User>("/api/users/me");
}

export function login(payload: LoginRequest) {
    return request<User>("/api/users/auth", jsonRequest(payload));
}

export function register(payload: RegisterRequest) {
    return request<User>("/api/users/register", jsonRequest(payload));
}

export function logout() {
    return request<void>("/api/users/logout", { method: "POST" });
}

export function changePassword(userId: string, currentPassword: string, password: string) {
    return request<void>(`/api/users/${userId}/password`, jsonRequest({
        current_password: currentPassword,
        password,
    }));
}

export function verifyEmail(token: string) {
    return request<void>("/api/users/verify-email", jsonRequest({ token }));
}