export type Profile = { login: string; nickname: string; has_totp: boolean };
export type TotpSetup = { otp_auth_url: string };
export type Capabilities = { permissions: string[] };
export type Instance = {
    id: string;
    icon: string;
    dir_name: string;
    display_name: string;
    version: string;
    description: string;
    loader: string;
    loader_version?: string | null
};
export type Configuration = {
    id: string;
    icon: string;
    dir_name: string;
    display_name: string;
    instance_id: string;
    hard_dirs: string[];
    soft_dirs: string[];
    files: Record<string, { sha1?: string; url?: string; _type?: string }>
};
export type GroupSummary = { name: string; weight: number };
export type Group = GroupSummary & {
    permissions: {
        key: string;
        allow: boolean;
        context: Record<string, string>;
        source_priority: number;
        expires_at?: string | null
    }[];
    members: string[];
    parents: string[]
};
export type Ban = {
    id: string;
    user_id: string;
    scope: string;
    starts_at: string;
    expires_at?: string | null;
    reason_code: string;
    staff_note?: string | null;
    created_by: string;
    created_at: string;
    revoked_by?: string | null;
    revoked_at?: string | null;
    revoke_reason?: string | null
};

let token = localStorage.getItem('rumary_access_token');
export const authToken = () => token;
export const setAuthToken = (value: string | null) => {
    token = value;
    value ? localStorage.setItem('rumary_access_token', value) : localStorage.removeItem('rumary_access_token');
};

async function refresh() {
    const r = await fetch('/api/v1/auth/refresh', {method: 'POST', credentials: 'include'});
    if (!r.ok) throw new Error('sessionExpired');
    const d = await r.json();
    setAuthToken(d.access_token);
}

export async function api<T = unknown>(path: string, init: RequestInit = {}, retry = true): Promise<T> {
    const headers = new Headers(init.headers);
    if (init.body && !headers.has('Content-Type')) headers.set('Content-Type', 'application/json');
    if (token) headers.set('Authorization', `Bearer ${token}`);
    const r = await fetch(path, {...init, headers, credentials: 'include'});
    if (r.status === 401 && retry && !path.startsWith('/api/v1/auth/')) {
        try {
            await refresh();
            return api<T>(path, init, false);
        } catch {
            setAuthToken(null);
            throw new Error('authorizationRequired');
        }
    }
    if (!r.ok) {
        let message = `requestFailed:${r.status}`;
        try {
            const d = await r.json();
            message = d.message || d.error || message;
        } catch {
        }
        throw new Error(message);
    }
    if (r.status === 204) return undefined as T;
    return r.json().catch(() => undefined as T);
}

export async function apiBlob(path: string, init: RequestInit = {}, retry = true): Promise<Blob> {
    const headers = new Headers(init.headers);
    if (token) headers.set('Authorization', `Bearer ${token}`);
    const r = await fetch(path, {...init, headers, credentials: 'include'});
    if (r.status === 401 && retry && !path.startsWith('/api/v1/auth/')) {
        try {
            await refresh();
            return apiBlob(path, init, false);
        } catch {
            setAuthToken(null);
            throw new Error('authorizationRequired');
        }
    }
    if (!r.ok) {
        let message = `requestFailed:${r.status}`;
        try {
            const d = await r.json();
            message = d.message || d.error || message;
        } catch {
        }
        throw new Error(message);
    }
    return r.blob();
}

export const json = (body: unknown, method = 'POST'): RequestInit => ({method, body: JSON.stringify(body)});
